# stage 1
The Complete Rendering Pipeline
```rust
pub fn build_cmd_buf(&self, allocator: Arc<StandardCommandBufferAllocator>) {

    // 3. Set viewport (required if you have dynamic viewport state)
    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: renderer.swapchain_image_size().map(|v| v as f32),
        depth_range: 0.0..=1.0,
    };
    cb.set_viewport(0, [viewport].into_iter().collect())
        .unwrap();

    // 4. Bind graphics pipeline (you need to create this!)
    cb.bind_pipeline_graphics(self.pipeline.clone())
        .unwrap();

    // 5. (Optional) Bind vertex buffers if you're using them
    // cb.bind_vertex_buffers(0, vertex_buffer.clone())
    //     .unwrap();

    // 6. (Optional) Bind descriptor sets for uniforms/textures
    // cb.bind_descriptor_sets(
    //     PipelineBindPoint::Graphics,
    //     pipeline.layout().clone(),
    //     0,
    //     descriptor_set.clone(),
    // )
    // .unwrap();

    // 7. (Optional) Push constants if you're using them
    // cb.push_constants(pipeline.layout().clone(), 0, push_data)
    //     .unwrap();

    // 8. Issue draw command
    unsafe {
        cb.draw(
            3,  // vertex_count (e.g., 3 for a triangle)
            1,  // instance_count
            0,  // first_vertex
            0,  // first_instance
        )
    }
    .unwrap();

    // 9. End render pass
    cb.end_render_pass(Default::default())
        .unwrap();

    // 10. Build the command buffer
    let command_buffer = cb.build().unwrap();

    // Now you need to execute this command buffer!
    // (See next section)
}
  ```
# stage 2. Creating pipeline

 1. First, Create Shaders

  Create two simple shader files:

  cube/src/vert.glsl:
  #version 450

  layout(location = 0) out vec3 fragColor;

  void main() {
      // Simple triangle vertices (no vertex buffer)
      vec2 positions[3] = vec2[](
          vec2(0.0, -0.5),
          vec2(0.5, 0.5),
          vec2(-0.5, 0.5)
      );

      vec3 colors[3] = vec3[](
          vec3(1.0, 0.0, 0.0),  // Red
          vec3(0.0, 1.0, 0.0),  // Green
          vec3(0.0, 0.0, 1.0)   // Blue
      );

      gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
      fragColor = colors[gl_VertexIndex];
  }

  cube/src/frag.glsl:
  #version 450

  layout(location = 0) in vec3 fragColor;
  layout(location = 0) out vec4 outColor;

  void main() {
      outColor = vec4(fragColor, 1.0);
  }

  2. Update Your RenderContext Struct

  use vulkano::pipeline::{GraphicsPipeline, graphics::viewport::Viewport};

  struct RenderContext {
      window_ctx: VulkanoWindows,
      id: WindowId,
      render_pass: Arc<RenderPass>,
      framebuffers: Vec<Arc<Framebuffer>>,  // ✓ You have this
      pipeline: Arc<GraphicsPipeline>,      // ← Uncomment and add this!
  }

  3. Add Pipeline Creation

  Add this to your RenderContextBuilder:

  impl RenderContextBuilder {
      pub fn with_pipeline(mut self) -> Self {
          // Load shaders
          mod vs {
              vulkano_shaders::shader! {
                  ty: "vertex",
                  path: "src/vert.glsl",
              }
          }
          mod fs {
              vulkano_shaders::shader! {
                  ty: "fragment",
                  path: "src/frag.glsl",
              }
          }

          let vs = vs::load(self.basic_cntx.device().clone())
              .unwrap()
              .entry_point("main")
              .unwrap();
          let fs = fs::load(self.basic_cntx.device().clone())
              .unwrap()
              .entry_point("main")
              .unwrap();

          let vertex_input_state = VertexInputState::default();
          let stages = [
              PipelineShaderStageCreateInfo::new(vs),
              PipelineShaderStageCreateInfo::new(fs),
          ];

          let layout = PipelineLayout::new(
              self.basic_cntx.device().clone(),
              PipelineLayoutCreateInfo::default(),
          )
          .unwrap();

          let subpass = Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap();

          let pipeline = GraphicsPipeline::new(
              self.basic_cntx.device().clone(),
              None,
              GraphicsPipelineCreateInfo {
                  stages: stages.into_iter().collect(),
                  vertex_input_state: Some(vertex_input_state),
                  input_assembly_state: Some(InputAssemblyState {
                      topology: PrimitiveTopology::TriangleList,
                      ..Default::default()
                  }),
                  viewport_state: Some(ViewportState::default()),
                  rasterization_state: Some(RasterizationState::default()),
                  multisample_state: Some(MultisampleState::default()),
                  color_blend_state: Some(ColorBlendState::with_attachment_states(
                      subpass.num_color_attachments(),
                      ColorBlendAttachmentState::default(),
                  )),
                  dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                  subpass: Some(subpass.into()),
                  ..GraphicsPipelineCreateInfo::layout(layout)
              },
          )
          .unwrap();

          self.pipeline = Some(pipeline);
          self
      }
  }
# stage 3. Executing and presenting.

 After building the command buffer, you need to execute it:

  pub fn draw(&mut self) {
      let renderer = self.window_ctx.get_renderer_mut(self.id).unwrap();

      // Acquire image
      let acquire_future = match renderer.acquire(None, |_| {}) {
          Ok(f) => f,
          Err(_) => return,
      };

      // Build command buffer (using your build_cmd_buf method)
      let command_buffer = self.build_cmd_buf(/*allocator*/);

      // Execute and present
      let after_future = acquire_future
          .then_execute(renderer.graphics_queue().clone(), command_buffer)
          .unwrap()
          .boxed();

      renderer.present(after_future, true);
  }
