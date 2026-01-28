#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 fragUV;

layout(set = 0, binding = 0) uniform Data {
    mat4 ortho;
} uniforms;

void main() {
    gl_Position = uniforms.ortho * vec4(position, 1.0);
    fragUV = uv;
}
