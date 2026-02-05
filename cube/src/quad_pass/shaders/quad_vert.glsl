#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 fragUV;

layout(set = 0, binding = 0) uniform Data {
    vec2 screen_size;
} uniforms;

void main() {
    // Convert pixel coordinates to NDC.
    // Y is negated because NDC Y=-1 is top in Vulkan but this app
    // uses the OpenGL convention (Y=-1 bottom) via its projection matrices.
    vec2 ndc = position.xy / uniforms.screen_size * 2.0 - 1.0;
    ndc.y = -ndc.y;
    gl_Position = vec4(ndc, 0.0, 1.0);
    fragUV = uv;
}
