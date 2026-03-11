function sendLog(url) {
  fetch("http://127.0.0.1:7777/log", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      url: url,
      timestamp: Date.now(),
    }),
  }).catch((err) => console.log("log failed", err));
}

// Fires on normal page loads
browser.webNavigation.onCompleted.addListener((details) => {
  if (details.frameId !== 0) return;
  sendLog(details.url);
});

// Fires when SPA updates URL (YouTube, Twitter, etc.)
browser.webNavigation.onHistoryStateUpdated.addListener((details) => {
  if (details.frameId !== 0) return;
  sendLog(details.url);
});
