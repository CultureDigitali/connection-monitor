export function applyOfficialLogo(images, source, altText) {
    for (const image of images) {
        image.src = source;
        image.alt = altText;
    }
}
