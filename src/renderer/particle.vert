#version 330 core

layout(location = 0) in vec3 particlePosition;
out float particleVelocity;

void main() {
    gl_Position = vec4(particlePosition.xy, 0.0, 1.0);
    particleVelocity = particlePosition.z;
}
