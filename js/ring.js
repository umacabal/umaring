document.addEventListener('DOMContentLoaded', async () => {
    const webringContainer = document.getElementById('umaring');
    if (!webringContainer) {
        console.error('UMass Amherst webring container not found.');
        return;
    }

    try {
        const scriptTag = document.getElementById('umaring_js');
        if (!scriptTag) {
            console.error('UMass Amherst webring script tag not found.');
            return;
        }

        const memberId = new URL(scriptTag.src).searchParams.get('id');
        if (!memberId) {
            console.error('Member ID not specified in script tag.');
            return;
        }

        const response = await fetch(`https://umaring.hamy.cc/${memberId}`);
        if (!response.ok) {
            console.error('Failed to fetch UMass Amherst webring data.');
            return;
        }

        const data = await response.json();
        const { prev, member, next } = data;

        webringContainer.innerHTML = `
            <a href="${prev.url}" id="umaring_prev">${prev.name}</a> <-
            <a href="https://github.com/umaring/umaring">UMass Ring</a> ->
            <a href="${next.url}" id="umaring_next">${next.name}</a>
        `;
    } catch (error) {
        console.error('Error fetching UMass Amherst webring data:', error);
    }
});
