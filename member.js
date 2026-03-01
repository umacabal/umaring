(function () {
  const prev = PREV_DATA_HERE;
  const next = NEXT_DATA_HERE;

  function init() {
    const scriptUrl = new URL(document.currentScript.src);
    const mode = scriptUrl.searchParams.get("mode");

    if (mode === "link") {
      const webringPrev = document.getElementById("umaring_prev");
      const webringNext = document.getElementById("umaring_next");
      if (!webringPrev || !webringNext) {
        console.error("UMass Amherst webring mode link requires elements #umaring_prev and #umaring_next.");
        return;
      }
      webringPrev.href = prev.url;
      webringPrev.textContent = prev.name;
      webringNext.href = next.url;
      webringNext.textContent = next.name;
    } else {
      const el = document.getElementById("umaring");
      if (!el) {
        console.error("UMass Amherst webring container not found.");
        return;
      }
      el.innerHTML = `
        <a href="${prev.url}" id="umaring_prev">${prev.name}</a> <-
        <a href="https://github.com/umaring/umaring">UMass Ring</a> ->
        <a href="${next.url}" id="umaring_next">${next.name}</a>
      `;
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
