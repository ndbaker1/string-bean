/**
 * Compute anchors for a circle
 * @param {number} anchor_count 
 * @param {number} width 
 * @param {number} height 
 * @param {number} radius 
 * @returns {[number, number][]}
 */
export function circle_anchors(anchor_count, width, height, radius) {
    const x_mid = width / 2;
    const y_mid = height / 2;

    return Array.from({ length: anchor_count })
        .map((_, anchor) => (anchor * 2 * Math.PI) / anchor_count)
        .map((angle) => [
            x_mid + radius * Math.cos(angle),
            y_mid + radius * Math.sin(angle),
        ]);
}

/**
 * Compute anchors for a rectangle. Starts in the top left.
 * @param {number} anchor_count 
 * @param {number} width 
 * @param {number} height 
 * @returns {[number, number][]}
 */
export function rectangle_anchors(anchor_count, width, height) {
    /** @type {[number, number][]} */
    const anchors = [[0, 0]];

    const totalLength = width * 2 + height * 2;
    const gapLength = totalLength / anchor_count;

    let currentLength = 0;

    for (let i = 1; i < anchor_count; i++) {
        currentLength += gapLength;

        if (currentLength < width) {
            // walk top left to right
            anchors.push([currentLength, 0]);
        } else if (currentLength < width + height) {
            // walk right top to bottom
            anchors.push([width, currentLength - (width)]);
        } else if (currentLength < width * 2 + height) {
            // walk bottom right to left
            anchors.push([2 * width - (currentLength - height), height]);
        } else {
            // walk left bottom to top
            anchors.push([0, height + 2 * width - (currentLength - height)]);
        }
    }

    return anchors;
}