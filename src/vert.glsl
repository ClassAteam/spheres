#version 450

layout(location = 0) in vec3 position;
layout(push_constant) uniform PushConstants {
    float rotation_angle;
} pc;

void main() {
    float cos_a = cos(pc.rotation_angle);
    float sin_a = sin(pc.rotation_angle);
    
    // 2D rotation matrix applied to x,y coordinates
    vec2 rotated_pos = vec2(
        position.x * cos_a - position.y * sin_a,
        position.x * sin_a + position.y * cos_a
    );
    
    gl_Position = vec4(rotated_pos, position.z, 1.0); 
}
