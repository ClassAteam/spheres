#version 450

layout(location = 0) in vec3 position;
layout(location = 0) out vec3 fragColor;

layout(set = 0, binding = 0) uniform Data {
    mat4 mvp;
} uniforms;

void main() {

  vec3 colors[8] = vec3[](
      vec3(1.0, 0.0, 0.0),
      vec3(0.0, 1.0, 0.0),
      vec3(0.0, 0.0, 1.0),
      vec3(1.0, 1.0, 0.0),
      vec3(1.0, 0.0, 1.0),
      vec3(0.0, 1.0, 1.0),
      vec3(1.0, 1.0, 1.0),
      vec3(0.5, 0.5, 0.5)
  );

  gl_Position = uniforms.mvp * vec4(position, 1.0);
  fragColor = colors[gl_VertexIndex];
}
