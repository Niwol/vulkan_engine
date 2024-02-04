#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_texture_coord;

layout(location = 0) out vec3 normal;
layout(location = 1) out vec2 tex_coords;

layout(push_constant) uniform MVP 
{ 
    mat4 model; 
    mat4 view;
    mat4 proj;
} mvp;

void main() {
    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(in_position, 1.0);
    normal = mat3(transpose(inverse(mvp.model))) * in_normal;
    tex_coords = in_texture_coord;
}
