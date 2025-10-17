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
     if (gl_VertexIndex == 0) {
          // Center vertex
          gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
      } else {
          // Circumference vertices (1, 2, 3, ...)
          float angle = 2.0 * 3.14159 * float(gl_VertexIndex - 1) / (pc.segments_per_circle - 1);
          vec2 pos = vec2(
              pc.circle_radius * cos(angle + pc.rotation_angle),
              pc.circle_radius * sin(angle + pc.rotation_angle)
          );
          pos.x /= pc.aspect_ratio;
          gl_Position = vec4(pos, 0.0, 1.0);
      }   
}
