#version 450

layout(push_constant) uniform PushConstants {
    float rotation_angle;
    float aspect_ratio;
    float circle_radius;
    float segments_per_circle;
} pc;

void main() {
    // Generate a single circle for now
    float angle = 2.0 * 3.14159265359 * float(gl_VertexIndex) / pc.segments_per_circle;
    
    // Add rotation
    float rotated_angle = angle + pc.rotation_angle;
    
    // Generate circle position  
    vec2 pos = vec2(
        pc.circle_radius * cos(rotated_angle),
        pc.circle_radius * sin(rotated_angle)
    );
    
    // Apply aspect ratio correction
    pos.x /= pc.aspect_ratio;
    
    gl_Position = vec4(pos, 0.0, 1.0);
}
