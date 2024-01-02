#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;

layout(location = 0) out vec3 normal;

layout(push_constant) uniform constants 
{ 
    mat4 model; 
    mat4 view;
    mat4 proj;
} push_constants;

void main() {
    gl_Position = push_constants.proj * push_constants.view * push_constants.model * vec4(in_position, 1.0);
    normal = mat3(transpose(inverse(push_constants.model))) * in_normal;
}
