/**
 * Universal Browser Sync Extension — Popup Script
 * Shows cookie metadata for the current tab.
 */

async function init() {
  const main = document.getElementById("main");

  // Get active tab
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab) {
    main.innerHTML = '<div class="loading">No active tab.</div>';
    return;
  }

  let url;
  try {
    url = new URL(tab.url);
  } catch {
    main.innerHTML = '<div class="loading">Cannot inspect this page.</div>';
    return;
  }

  // Protected pages
  if (url.protocol === "chrome:" || url.protocol === "about:") {
    main.innerHTML = '<div class="loading">Cannot inspect browser pages.</div>';
    return;
  }

  // Request cookies from background
  const cookies = await new Promise((resolve) => {
    chrome.runtime.sendMessage(
      { action: "GET_COOKIES", tabId: tab.id },
      (resp) => resolve(resp ?? [])
    );
  });

  const total = cookies.length;
  const authCount = cookies.filter((c) => c.is_auth_likely).length;
  const sessionCount = cookies.filter((c) => c.is_session).length;
  const secureCount = cookies.filter((c) => c.secure).length;

  main.innerHTML = `
    <div class="section-title">Current Page</div>
    <div style="font-size:11px; color:var(--muted); margin-bottom:8px; word-break:break-all;">${url.hostname}</div>

    <div class="section-title">Cookie Summary</div>
    <div style="background:var(--bg2); border:1px solid var(--border); border-radius:8px; padding:0 8px; margin-bottom:4px;">
      <div class="stat-row">
        <span class="stat-label">Total cookies</span>
        <span class="stat-value">${total}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Auth / session</span>
        <span class="stat-value">
          <span class="badge badge-amber">${authCount} auth</span>
        </span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Session-only</span>
        <span class="stat-value">${sessionCount}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Secure flag</span>
        <span class="stat-value">
          <span class="badge ${secureCount === total && total > 0 ? "badge-green" : "badge-amber"}">${secureCount}/${total}</span>
        </span>
      </div>
    </div>

    ${total > 0 ? `
    <div class="section-title">Cookies (${Math.min(total, 20)} of ${total})</div>
    <div id="cookie-list">
      ${cookies.slice(0, 20).map((c) => `
        <div class="cookie-row">
          <div style="flex:1; min-width:0;">
            <div class="cookie-name">${escHtml(c.name)}</div>
            <div class="cookie-domain">${escHtml(c.domain)}</div>
          </div>
          <div style="display:flex;gap:4px;align-items:center;flex-shrink:0;">
            ${c.is_auth_likely ? '<span class="badge badge-amber">auth</span>' : ""}
            ${c.secure ? '<span class="badge badge-green">sec</span>' : ""}
            ${c.is_session ? '<span class="badge" style="background:rgba(148,163,184,.1);color:var(--muted)">sess</span>' : ""}
          </div>
        </div>
      `).join("")}
    </div>
    ` : '<div style="color:var(--muted);text-align:center;padding:12px 0;font-size:12px;">No cookies found for this page.</div>'}

    <button class="btn" id="open-app">Open Universal Browser Sync ↗</button>
  `;

  document.getElementById("open-app")?.addEventListener("click", () => {
    // Attempt to open the desktop app via a custom URI scheme
    // The Tauri app registers ubs:// on install
    window.open("ubs://open", "_blank");
  });
}

function escHtml(str) {
  return String(str)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

init().catch((e) => {
  document.getElementById("main").innerHTML =
    `<div class="loading" style="color:#ef4444;">Error: ${e.message}</div>`;
});
