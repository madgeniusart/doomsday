/** User-initiated, metadata-only cookie inspector. */

async function sendBackground(message) {
  return chrome.runtime.sendMessage(message);
}

async function init() {
  const main = document.getElementById("main");
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id || !tab.url) {
    main.textContent = "No inspectable active tab.";
    return;
  }

  let pageUrl;
  try {
    pageUrl = new URL(tab.url);
  } catch {
    main.textContent = "Cannot inspect this page.";
    return;
  }
  if (!["http:", "https:"].includes(pageUrl.protocol)) {
    main.textContent = "Only ordinary HTTP and HTTPS pages can be inspected.";
    return;
  }

  const originPattern = `${pageUrl.protocol}//${pageUrl.hostname}/*`;
  const hasPermission = await chrome.permissions.contains({ origins: [originPattern] });
  if (!hasPermission) {
    main.innerHTML = `
      <div class="section-title">Site permission required</div>
      <div class="notice">UBS can inspect cookie <strong>metadata only</strong> for this site. Cookie values, passwords, and page storage are never collected.</div>
      <div class="site-name">${escHtml(pageUrl.hostname)}</div>
      <button class="btn" id="grant-site">Allow for this site</button>
      <div class="fine-print">You can revoke this permission in your browser's extension settings.</div>
    `;
    document.getElementById("grant-site")?.addEventListener("click", async () => {
      const granted = await chrome.permissions.request({ origins: [originPattern] });
      if (granted) init();
      else showError("Permission was not granted.");
    });
    return;
  }

  main.innerHTML = '<div class="loading">Reading cookie metadata…</div>';
  const response = await sendBackground({ action: "GET_COOKIE_METADATA", tabId: tab.id });
  if (!response?.ok) {
    showError(response?.error || "Could not inspect this page.");
    return;
  }
  renderCookies(main, pageUrl.hostname, response.cookies || [], originPattern);
  const bridge = await sendBackground({ action: "GET_BRIDGE_STATUS" }).catch(() => ({ status: "unknown" }));
  const bridgeLine = document.createElement("div");
  bridgeLine.className = "notice";
  bridgeLine.textContent = `Desktop bridge: ${bridge?.status || "unknown"}`;
  main.appendChild(bridgeLine);
}

function renderCookies(main, hostname, cookies, originPattern) {
  const total = cookies.length;
  const authCount = cookies.filter((cookie) => cookie.is_auth_likely).length;
  const sessionCount = cookies.filter((cookie) => cookie.is_session).length;
  const secureCount = cookies.filter((cookie) => cookie.secure).length;
  main.innerHTML = `
    <div class="section-title">Current site</div>
    <div class="site-name">${escHtml(hostname)}</div>
    <div class="section-title">Cookie metadata</div>
    <div class="summary">
      <div class="stat-row"><span class="stat-label">Total cookies</span><span class="stat-value">${total}</span></div>
      <div class="stat-row"><span class="stat-label">Likely authentication</span><span class="stat-value">${authCount}</span></div>
      <div class="stat-row"><span class="stat-label">Session-only</span><span class="stat-value">${sessionCount}</span></div>
      <div class="stat-row"><span class="stat-label">Secure</span><span class="stat-value">${secureCount}/${total}</span></div>
    </div>
    ${total ? `
      <div class="section-title">Cookie names (${Math.min(total, 20)} of ${total})</div>
      <div id="cookie-list">${cookies.slice(0, 20).map((cookie) => `
        <div class="cookie-row">
          <div class="cookie-info"><div class="cookie-name">${escHtml(cookie.name)}</div><div class="cookie-domain">${escHtml(cookie.domain)}</div></div>
          <div class="flags">${cookie.is_auth_likely ? '<span class="badge badge-amber">auth</span>' : ""}${cookie.secure ? '<span class="badge badge-green">secure</span>' : ""}${cookie.is_session ? '<span class="badge">session</span>' : ""}</div>
        </div>`).join("")}</div>
    ` : '<div class="empty">No cookies found for this site.</div>'}
    <div class="notice">This helper never reads or exports cookie values. Authentication sessions cannot be transferred by this metadata view.</div>
    <button class="btn secondary" id="revoke-site">Revoke this site permission</button>
  `;
  document.getElementById("revoke-site")?.addEventListener("click", async () => {
    await chrome.permissions.remove({ origins: [originPattern] }).catch(() => {});
    init();
  });
}

function showError(message) {
  const main = document.getElementById("main");
  main.innerHTML = `<div class="error">${escHtml(message)}</div><button class="btn secondary" id="retry">Retry</button>`;
  document.getElementById("retry")?.addEventListener("click", init);
}

function escHtml(value) {
  return String(value).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

init().catch((error) => showError(error?.message || "Unexpected extension error."));
