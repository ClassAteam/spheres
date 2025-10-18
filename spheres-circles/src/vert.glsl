#version 450

layout(push_constant) uniform PushConstants {
    float rotation_angle;
    float aspect_ratio;
    float circle_radius;
    float segments_per_circle;
} pc;

layout(location = 0) out flat vec3 vertex_color;

void main() {
    if (gl_VertexIndex == 0) {
        // Center vertex
        gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        vertex_color = vec3(1.0, 1.0, 1.0); // Will be overridden by triangle color
    } else {
        // Circumference vertices
        float angle = 2.0 * 3.14159265359 * float(gl_VertexIndex - 1) / (pc.segments_per_circle - 1);
        float rotated_angle = angle + pc.rotation_angle;
        
        // Generate position
        vec2 pos = vec2(
            pc.circle_radius * cos(rotated_angle),
            pc.circle_radius * sin(rotated_angle)
        );
        
        // Apply aspect ratio correction
        pos.x /= pc.aspect_ratio;
        gl_Position = vec4(pos, 0.0, 1.0);
        
        // Each triangle gets a unique color based on triangle number
        int triangle_id = gl_VertexIndex - 1;  // Triangle 0, 1, 2, etc.
        float hue = float(triangle_id) / (pc.segments_per_circle - 1);
        
        // Generate distinct colors for each triangle
        vertex_color = vec3(
            sin(hue * 6.28318 * 3.0) * 0.5 + 0.5,       // Red
            sin(hue * 6.28318 * 3.0 + 2.094) * 0.5 + 0.5, // Green
            sin(hue * 6.28318 * 3.0 + 4.188) * 0.5 + 0.5  // Blue
        );
    }
}
