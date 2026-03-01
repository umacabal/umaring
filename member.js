(function () {
  const prev = PREV_DATA_HERE;
  const next = NEXT_DATA_HERE;

  function init() {
    const el = document.getElementById("umaring");
    if (!el) {
      console.error("UMass Amherst webring: #umaring element not found.");
      return;
    }
    el.innerHTML = `
      <a href="${prev.url}" id="umaring_prev">${prev.name}</a> <-
      <a href="https://github.com/umaring/umaring">UMass Ring</a> ->
      <a href="${next.url}" id="umaring_next">${next.name}</a>
    `;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
