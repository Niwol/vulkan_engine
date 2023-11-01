#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_color;

layout(location = 0) out vec3 fragColor;

layout(binding = 0) uniform MVP {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

void main() {
    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(in_position, 1.0);
    fragColor = in_color;
}
