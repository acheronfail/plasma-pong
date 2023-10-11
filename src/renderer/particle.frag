#version 330 core

out vec4 FragColor;

void main() {
    // gl_PointCoord provides the coordinate within the point sprite, ranging from (0,0) to (1,1).
    // We calculate the distance from the center of the point.
    float dist = length(gl_PointCoord - vec2(0.5));

    // If the distance is greater than 0.5, discard the fragment.
    if (dist > 0.5) {
        discard;
    }

    // Otherwise, set the fragment color as desired.
    FragColor = vec4(0.0, 1.0, 0.0, 1.0);
}