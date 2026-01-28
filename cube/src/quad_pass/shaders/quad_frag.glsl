#version 450

layout(location = 0) in vec2 fragUV;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D texSampler;

void main() {
    // Sample the grayscale texture
    float gray = texture(texSampler, fragUV).r;

    // Convert to RGB (white text on transparent background)
    outColor = vec4(vec3(gray), 1.0);
}
