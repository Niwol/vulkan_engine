#version 450

layout(location = 0) in vec3 normal;
layout(location = 1) in vec2 tex_coords;

layout(location = 0) out vec4 out_color;

layout(binding = 0) uniform Material
{
    vec3 color;
} material;

void main() {
    vec3 ligh_dir = normalize(vec3(0.2, -1.0, -0.3));
    float attenuation = max(dot(-ligh_dir, normal), 0.0);
    out_color = vec4(material.color * attenuation, 1.0);
}
