(function () {
  const RING = RING_DATA_HERE;

  const scriptUrl = new URL(document.currentScript.src);
  const scriptParams = {
    id: scriptUrl.searchParams.get("id"),
    mode: scriptUrl.searchParams.get("mode"),
  };

  function initializeWebring() {
    try {
      const { id, mode } = scriptParams;

      if (!id) {
        console.error("UMass Amherst webring: missing id parameter in script src.");
        return;
      }

      const index = RING.findIndex((m) => m.id === id);
      if (index === -1) {
        console.error("UMass Amherst webring: member not found:", id);
        return;
      }

      const prev = RING[(index - 1 + RING.length) % RING.length];
      const next = RING[(index + 1) % RING.length];

      switch (mode) {
        case "link":
          const webringPrev = document.getElementById("umaring_prev");
          const webringNext = document.getElementById("umaring_next");

          if (!webringPrev || !webringNext) {
            console.error(
              "UMass Amherst webring mode link requires elements #umaring_prev and #umaring_next.",
            );
            return;
          }

          webringPrev.href = prev.url;
          webringPrev.textContent = prev.name;
          webringNext.href = next.url;
          webringNext.textContent = next.name;
          break;
        default:
          const webringContainer = document.getElementById("umaring");

          if (!webringContainer) {
            console.error("UMass Amherst webring container not found.");
            return;
          }

          webringContainer.innerHTML = `
            <a href="${prev.url}" id="umaring_prev">${prev.name}</a> <-
            <a href="https://github.com/umaring/umaring">UMass Ring</a> ->
            <a href="${next.url}" id="umaring_next">${next.name}</a>
          `;
          break;
      }
    } catch (error) {
      console.error("Error initializing UMass Amherst webring data:", error);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializeWebring);
  } else {
    initializeWebring();
  }
})();
