/**
 * Universal Browser Sync — extension service worker.
 * Cookie values and web-storage contents are never collected or transmitted.
 * Site access is optional and must be granted by the user for each origin.
 */

const AUTH_COOKIE_PATTERNS = [
  "session", "sess", "auth", "token", "jwt", "login", "sid",
  "user_id", "userid", "account", "csrftoken", "xsrf",
  "access_token", "refresh_token", "id_token",
];

function isAuthCookie(name) {
  const lower = String(name || "").toLowerCase();
  return AUTH_COOKIE_PATTERNS.some((pattern) => lower.includes(pattern));
}

function toMetadata(cookie) {
  return {
    domain: cookie.domain,
    name: cookie.name,
    path: cookie.path,
    secure: Boolean(cookie.secure),
    http_only: Boolean(cookie.httpOnly),
    same_site: cookie.sameSite ?? "unspecified",
    expires: cookie.expirationDate ? Math.floor(cookie.expirationDate) : null,
    is_session: Boolean(cookie.session),
    is_auth_likely: isAuthCookie(cookie.name),
  };
}

async function getTabCookieMetadata(tabId) {
  const tab = await chrome.tabs.get(tabId);
  if (!tab?.url) throw new Error("The active tab has no inspectable URL.");
  const url = new URL(tab.url);
  if (!["http:", "https:"].includes(url.protocol)) {
    throw new Error("Only ordinary HTTP and HTTPS pages can be inspected.");
  }
  const origin = `${url.protocol}//${url.hostname}/*`;
  const permitted = await chrome.permissions.contains({ origins: [origin] });
  if (!permitted) throw new Error("Site permission has not been granted.");
  const cookies = await chrome.cookies.getAll({ domain: url.hostname });
  return cookies.map(toMetadata);
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.action === "PING") {
    sendResponse({ status: "ok", version: "1.1.0", mode: "metadata_only" });
    return false;
  }
  if (message?.action === "GET_COOKIE_METADATA") {
    getTabCookieMetadata(message.tabId)
      .then((cookies) => sendResponse({ ok: true, cookies }))
      .catch((error) => sendResponse({ ok: false, error: error.message }));
    return true;
  }
  return false;
});

chrome.runtime.onInstalled.addListener(({ reason }) => {
  if (reason === "install") {
    console.info("[UBS] Helper installed. Site access is requested only when you choose to inspect a page.");
  }
});

// Native-messaging bridge: only health and aggregate profile counts are sent.
// Passwords, cookie values, browsing URLs, and authentication tokens are never sent.
const NATIVE_HOST = "com.universal-browser-sync.bridge";
let nativePort = null;
let bridgeStatus = "not_installed";

function connectNativeBridge() {
  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST);
    nativePort.onMessage.addListener((message) => {
      bridgeStatus = message?.ok ? "connected" : "error";
      chrome.storage.local.set({ ubsBridgeStatus: bridgeStatus, ubsBridgeInfo: message || null });
    });
    nativePort.onDisconnect.addListener(() => {
      const error = chrome.runtime.lastError?.message || "Native host unavailable";
      bridgeStatus = "disconnected";
      nativePort = null;
      chrome.storage.local.set({ ubsBridgeStatus: bridgeStatus, ubsBridgeError: error });
    });
    nativePort.postMessage({ action: "PING" });
  } catch (error) {
    bridgeStatus = "not_installed";
    chrome.storage.local.set({ ubsBridgeStatus: bridgeStatus, ubsBridgeError: String(error) });
  }
}

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.action === "GET_BRIDGE_STATUS") {
    chrome.storage.local.get(["ubsBridgeStatus", "ubsBridgeError"], (result) => {
      sendResponse({ status: result.ubsBridgeStatus || bridgeStatus, error: result.ubsBridgeError || null });
    });
    return true;
  }
  if (message?.action === "SEND_PROFILE_SUMMARY") {
    if (!nativePort) {
      sendResponse({ ok: false, error: "Native bridge is not connected" });
      return false;
    }
    nativePort.postMessage({
      action: "PROFILE_SUMMARY",
      browser: message.browser,
      profileName: message.profileName,
      counts: {
        bookmarks: Number(message.counts?.bookmarks || 0),
        history: Number(message.counts?.history || 0),
        extensions: Number(message.counts?.extensions || 0),
      },
    });
    sendResponse({ ok: true });
    return false;
  }
  return false;
});

connectNativeBridge();
