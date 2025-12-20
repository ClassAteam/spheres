#version 450

layout(push_constant) uniform PushConstants {
    float rotation_x;
    float rotation_y;
    float rotation_z;
    float sphere_radius;
    float segments;
    float rings;
} pc;

layout(location = 0) out vec3 vertex_color;

void main() {
    // Create a simple triangle in 3D space
    vec3 position;
    
    if (gl_VertexIndex == 0) {
        position = vec3(0.0, 0.5, 0.0);    // Top vertex
        vertex_color = vec3(1.0, 0.0, 0.0); // Red
    } else if (gl_VertexIndex == 1) {
        position = vec3(-0.5, -0.5, 0.0);  // Bottom left
        vertex_color = vec3(0.0, 1.0, 0.0); // Green
    } else { // gl_VertexIndex == 2
        position = vec3(0.5, -0.5, 0.0);   // Bottom right
        vertex_color = vec3(0.0, 0.0, 1.0); // Blue
    }
    
    // Apply 3D rotations
    float cos_rx = cos(pc.rotation_x);
    float sin_rx = sin(pc.rotation_x);
    float cos_ry = cos(pc.rotation_y);  
    float sin_ry = sin(pc.rotation_y);
    
    // Rotation around X axis (pitch)
    float y1 = position.y * cos_rx - position.z * sin_rx;
    float z1 = position.y * sin_rx + position.z * cos_rx;
    
    // Rotation around Y axis (yaw)
    float x2 = position.x * cos_ry + z1 * sin_ry;
    float z2 = -position.x * sin_ry + z1 * cos_ry;
    
    gl_Position = vec4(x2, y1, z2, 1.0);
}
