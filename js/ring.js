document.addEventListener("DOMContentLoaded", async () => {
  try {
    const scriptTag = document.getElementById("umaring_js");
    if (!scriptTag) {
      console.error("UMass Amherst webring script tag not found.");
      return;
    }

    const memberId = new URL(scriptTag.src).searchParams.get("id");
    if (!memberId) {
      console.error("Member ID not specified in script tag.");
      return;
    }

    const response = await fetch(`https://umaring.mkr.cx/${memberId}`);
    if (!response.ok) {
      console.error("Failed to fetch UMass Amherst webring data.");
      return;
    }

    const data = await response.json();
    const { prev, next } = data;

    const mode = new URL(scriptTag.src).searchParams.get("mode");
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

        document.getElementById("umaring_prev").href = prev.url;
        document.getElementById("umaring_prev").textContent = prev.name;
        document.getElementById("umaring_next").href = next.url;
        document.getElementById("umaring_next").textContent = next.name;
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
    console.error("Error fetching UMass Amherst webring data:", error);
  }
});
