async function load() {
  const loadingEl = document.getElementById("loading");
  const errorEl = document.getElementById("error");
  const listEl = document.getElementById("list");

  loadingEl.classList.add("visible");

  try {
    const res = await fetch("http://127.0.0.1:7777/allowlist");
    if (!res.ok) throw new Error();
    const domains = await res.json();

    loadingEl.classList.remove("visible");

    if (!domains.length) {
      errorEl.textContent = "No sites have been approved yet.";
      errorEl.classList.add("visible");
      return;
    }

    domains.sort();
    listEl.classList.add("visible");

    domains.forEach((domain, i) => {
      const a = document.createElement("a");
      a.className = "item";
      a.href = "https://" + domain;
      a.target = "_blank";
      a.rel = "noopener";
      a.style.opacity = "0";
      a.style.animation = "appear 0.4s ease " + i * 50 + "ms forwards";

      const img = document.createElement("img");
      img.className = "favicon";
      img.src =
        "https://www.google.com/s2/favicons?domain=" + domain + "&sz=32";
      img.alt = "";
      img.onerror = function () {
        const ph = document.createElement("div");
        ph.className = "favicon-placeholder";
        this.replaceWith(ph);
      };

      const left = document.createElement("div");
      left.className = "item-left";
      left.appendChild(img);

      const name = document.createElement("span");
      name.className = "domain";
      name.textContent = domain;
      left.appendChild(name);

      const arrow = document.createElement("span");
      arrow.className = "arrow";
      arrow.textContent = "↗";

      a.appendChild(left);
      a.appendChild(arrow);
      listEl.appendChild(a);
    });
  } catch {
    loadingEl.classList.remove("visible");
    errorEl.classList.add("visible");
  }
}

load();
