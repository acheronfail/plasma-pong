#version 330 core

// vec2(vertex) vec2(center point)
layout(location = 0) in vec4 vertex;

out vec2 particlePosition;

void main() {
    particlePosition = vertex.zw;

    gl_Position = vec4(vertex.xy, 0.0, 1.0);
}
