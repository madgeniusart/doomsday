/**
 * Universal Browser Sync — Extension Background Service Worker
 *
 * Provides cookie metadata, tab info, and local storage snapshots
 * to the desktop application via the native messaging API.
 *
 * SECURITY: Cookie *values* for auth cookies are never exposed.
 * Only metadata (domain, name, flags, expiry) is transmitted.
 */

const AUTH_COOKIE_PATTERNS = [
  "session", "sess", "auth", "token", "jwt", "login",
  "sid", "user_id", "userid", "account", "csrftoken",
  "xsrf", "access_token", "refresh_token", "id_token",
];

function isAuthCookie(name) {
  const lower = name.toLowerCase();
  return AUTH_COOKIE_PATTERNS.some((p) => lower.includes(p));
}

/**
 * Get cookie metadata for the current active tab.
 * Returns sanitised CookieInfo objects (no values for auth cookies).
 */
async function getTabCookies(tabId) {
  const [tab] = await chrome.tabs.query({ id: tabId });
  if (!tab?.url) return [];

  const url = new URL(tab.url);
  const cookies = await chrome.cookies.getAll({ domain: url.hostname });

  return cookies.map((c) => ({
    domain: c.domain,
    name: c.name,
    path: c.path,
    secure: c.secure,
    http_only: c.httpOnly,
    same_site: c.sameSite ?? "unspecified",
    expires: c.expirationDate ? Math.floor(c.expirationDate) : null,
    is_session: c.session,
    is_auth_likely: isAuthCookie(c.name),
    // Value deliberately excluded for auth-likely cookies
    value: isAuthCookie(c.name) ? "[REDACTED]" : c.value,
  }));
}

/**
 * Collect localStorage / sessionStorage snapshot from a tab via content script.
 */
async function getTabStorage(tabId) {
  try {
    const results = await chrome.scripting.executeScript({
      target: { tabId },
      func: () => {
        const local = {};
        const session = {};
        for (let i = 0; i < localStorage.length; i++) {
          const k = localStorage.key(i);
          local[k] = localStorage.getItem(k);
        }
        for (let i = 0; i < sessionStorage.length; i++) {
          const k = sessionStorage.key(i);
          session[k] = sessionStorage.getItem(k);
        }
        return { local, session };
      },
    });
    return results?.[0]?.result ?? { local: {}, session: {} };
  } catch {
    return { local: {}, session: {} };
  }
}

// ── Message handler from popup / desktop app ─────────────────────────────────

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  const { action, tabId } = message;

  if (action === "GET_COOKIES") {
    getTabCookies(tabId).then(sendResponse).catch((e) =>
      sendResponse({ error: e.message })
    );
    return true; // async
  }

  if (action === "GET_STORAGE") {
    getTabStorage(tabId).then(sendResponse).catch((e) =>
      sendResponse({ error: e.message })
    );
    return true;
  }

  if (action === "GET_ALL_COOKIES") {
    chrome.cookies.getAll({}).then((cookies) => {
      const metadata = cookies.map((c) => ({
        domain: c.domain,
        name: c.name,
        path: c.path,
        secure: c.secure,
        http_only: c.httpOnly,
        same_site: c.sameSite ?? "unspecified",
        expires: c.expirationDate ? Math.floor(c.expirationDate) : null,
        is_session: c.session,
        is_auth_likely: isAuthCookie(c.name),
      }));
      sendResponse(metadata);
    });
    return true;
  }

  if (action === "PING") {
    sendResponse({ status: "ok", version: "1.0.0" });
    return false;
  }
});

// Extension installed / updated handler
chrome.runtime.onInstalled.addListener(({ reason }) => {
  if (reason === "install") {
    console.log("[UBS] Extension installed — Universal Browser Sync Helper ready.");
  }
});
