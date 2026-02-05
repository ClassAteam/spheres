#version 450

layout(location = 0) in vec2 fragUV;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D texSampler;

void main() {
    // Grayscale coverage value becomes alpha; color is white.
    // Blended with standard src-alpha blending in the pipeline.
    float alpha = texture(texSampler, fragUV).r;
    outColor = vec4(1.0, 1.0, 1.0, alpha);
}
