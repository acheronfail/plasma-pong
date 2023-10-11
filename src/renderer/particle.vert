#version 330 core

layout(location = 0) in vec2 particlePosition;

void main() {
    gl_Position = vec4(particlePosition, 0.0, 1.0);
}
