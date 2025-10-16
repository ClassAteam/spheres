#version 450

layout(location = 0) in vec3 position;
layout(push_constant) uniform PushConstants {
    float rotation_angle;
    float aspect_ratio;
} pc;

void main() {
    float cos_a = cos(pc.rotation_angle);
    float sin_a = sin(pc.rotation_angle);

    vec2 pos = position.xy;

    // For LineList: vertex 0,2 are pivots; vertex 1,3 rotate around them
    if (gl_VertexIndex == 1) {
        // Rotate vertex 1 around vertex 0 (top line)
        vec2 pivot = vec2(-0.2, 0.2);  // vertex 0 position
        vec2 relative_pos = pos - pivot;
        vec2 rotated = vec2(relative_pos.x * cos_a - relative_pos.y * sin_a,
                           relative_pos.x * sin_a + relative_pos.y * cos_a);
        pos = pivot + rotated;
    } else if (gl_VertexIndex == 3) {
        // Rotate vertex 3 around vertex 2 (bottom line)
        vec2 pivot = vec2(0.2, -0.2);   // vertex 2 position
        vec2 relative_pos = pos - pivot;
        vec2 rotated = vec2(relative_pos.x * cos_a - relative_pos.y * sin_a,
                           relative_pos.x * sin_a + relative_pos.y * cos_a);
        pos = pivot + rotated;
    }

    // Apply aspect ratio correction
    pos.x /= pc.aspect_ratio;
    
    gl_Position = vec4(pos, position.z, 1.0);
}
