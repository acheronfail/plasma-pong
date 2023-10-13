#version 330 core

in vec2 particlePosition;
out vec4 FragColor;

void main() {
    vec2 windowSize = vec2(1600, 900);
    vec2 particlePixelCoords = 0.5 * ((particlePosition + 1.0) * windowSize);

    float dist = distance(gl_FragCoord.xy, particlePixelCoords);
    float max_distance = sqrt((windowSize.x * windowSize.x) + (windowSize.y * windowSize.y));

    float pressure = 1 - clamp((dist / max_distance) * 50, 0, 1);

    FragColor = vec4(vec3(pressure), 1.0);
}
