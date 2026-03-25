const BLOCKED_PAGE = chrome.runtime.getURL("blocked.html");

// Guard: skip URLs we generated ourselves
function isSafeToCheck(url) {
  if (!url) return false;
  if (url.startsWith("chrome://")) return false;
  if (url.startsWith("chrome-extension://")) return false;
  if (url.startsWith("about:")) return false;
  if (url.startsWith("data:")) return false;
  return true;
}

async function checkWithBackend(url) {
  try {
    const res = await fetch("http://127.0.0.1:7777/firewall", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url, timestamp: Date.now() }),
    });
    const data = await res.json();
    console.log("[SENTRY] Check result:", data);
    return data.allowed;
  } catch (err) {
    console.log("[SENTRY] Backend unreachable", err);
    return false;
  }
}

async function handleNavigation(details) {
  if (details.frameId !== 0) return; // main frame only
  const url = details.url;

  if (!isSafeToCheck(url)) return; // skip our own redirects

  const allowed = await checkWithBackend(url);

  if (!allowed) {
    console.log("[SENTINEL] Blocked:", url);
    chrome.tabs.update(details.tabId, {
      url: `${BLOCKED_PAGE}?url=${encodeURIComponent(url)}`,
    });
  }
}

// Handles normal navigations
chrome.webNavigation.onBeforeNavigate.addListener(handleNavigation);

// Handles SPA navigations (YouTube, etc.)
chrome.webNavigation.onHistoryStateUpdated.addListener(handleNavigation);
