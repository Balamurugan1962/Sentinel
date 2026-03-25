const params = new URLSearchParams(location.search);
const url = params.get("url");
if (url) {
  try {
    document.getElementById("url-display").textContent = new URL(url).hostname;
  } catch {
    document.getElementById("url-display").textContent = url;
  }
}
