#version 450

layout(location = 0) out vec4 outColor;

void main() {
    float near = 0.1;
    float far = 100.0;

    float z = gl_FragCoord.z * 2.0 - 1.0;
    float depth = (2.0 * near * far) / (far + near - z * (far - near));
    depth /= far;
    outColor = vec4(vec3(depth), 1.0);
}
