/**
 * Universal Browser Sync — Content Script
 * Passively available for storage inspection when the popup requests it.
 * Does NOT run any code automatically beyond registering the message listener.
 */

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.action === "GET_PAGE_STORAGE") {
    try {
      const local = {};
      const session = {};

      for (let i = 0; i < localStorage.length; i++) {
        const k = localStorage.key(i);
        if (k) local[k] = localStorage.getItem(k);
      }
      for (let i = 0; i < sessionStorage.length; i++) {
        const k = sessionStorage.key(i);
        if (k) session[k] = sessionStorage.getItem(k);
      }

      sendResponse({ ok: true, local, session });
    } catch (e) {
      sendResponse({ ok: false, error: e.message });
    }
    return false;
  }
});
