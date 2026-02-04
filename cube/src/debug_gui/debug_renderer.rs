use std::sync::Arc;

use egui_winit_vulkano::{Gui, egui};
use vulkano::{image::view::ImageView, sync::GpuFuture};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::event_loop::ActiveEventLoop;

use crate::{
    counter::FpsCounter,
    cube_pass::{self, TransformState},
    render::RenderContext,
};

pub struct DebugRenderer {
    gui: Gui,
}

impl DebugRenderer {
    pub fn new(event_loop: &ActiveEventLoop, renderer: &VulkanoWindowRenderer) -> Self {
        let gui = Gui::new(
            event_loop,
            renderer.surface(),
            renderer.graphics_queue().clone(),
            renderer.swapchain_format(),
            egui_winit_vulkano::GuiConfig {
                is_overlay: true,
                allow_srgb_render_target: true,
                ..Default::default()
            },
        );

        Self { gui }
    }

    // pub fn update(&mut self, event: &WindowEvent) -> bool {
    //     self.gui.update(event)
    // }

    pub fn create_ui(
        &mut self,
        fps_counter: &FpsCounter,
        transform: &TransformState,
        aspect_ratio: f32,
    ) {
        self.gui.immediate_ui(|gui| {
            let ctx = gui.context();
            egui::Window::new("Debug Info")
                .default_pos(egui::pos2(10.0, 10.0))
                .resizable(false)
                .default_width(1000.0)
                .show(&ctx, |ui| {
                    Self::show_fps(ui, fps_counter);
                    ui.separator();

                    ui.horizontal(|ui| {
                        Self::show_transform_state(ui, transform);
                        ui.separator();
                        Self::show_vertices(ui, transform, aspect_ratio);
                    });
                });
        });
    }

    fn show_fps(ui: &mut egui::Ui, fps_counter: &FpsCounter) {
        ui.label(format!("FPS: {:.1}", fps_counter.fps()));
        ui.label(format!("Frame Time: {:.2} ms", fps_counter.frame_time_ms()));
    }

    fn show_transform_state(ui: &mut egui::Ui, transform: &TransformState) {
        ui.vertical(|ui| {
            ui.heading("Transform State");
            ui.label(format!("{:#?}", transform));
        });
    }

    fn show_vertices(ui: &mut egui::Ui, transform: &TransformState, aspect_ratio: f32) {
        ui.vertical(|ui| {
            ui.heading("Vertices (Transformed)");

            egui::ScrollArea::vertical()
                .id_salt("transformed_vertices")
                .show(ui, |ui| {
                    for (i, vertex) in cube_pass::POSITIONS.iter().enumerate() {
                        let transformed = transform.transform_vertex(vertex.position, aspect_ratio);
                        ui.label(format!(
                            "[{}] clip: [{:.3}, {:.3}, {:.3}, {:.3}] -> ndc: [{:.3}, {:.3}, {:.3}]",
                            i,
                            transformed.clip_space[0],
                            transformed.clip_space[1],
                            transformed.clip_space[2],
                            transformed.clip_space[3],
                            transformed.ndc[0],
                            transformed.ndc[1],
                            transformed.ndc[2]
                        ));
                    }
                });
        });
    }

    fn draw(
        &mut self,
        image_view: Arc<ImageView>,
        last_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        self.gui.draw_on_image(last_future, image_view)
    }

    pub fn draw_ui(
        &mut self,
        rdx: &RenderContext,
        fps_counter: &FpsCounter,
        transform: &TransformState,
        last_future: Box<dyn GpuFuture>,
        visible: bool,
    ) -> Box<dyn GpuFuture> {
        let window_id = rdx.id;

        // Only create UI content if visible
        if visible {
            let aspect_ratio = rdx
                .window_ctx
                .get_renderer(window_id)
                .unwrap()
                .aspect_ratio();

            self.create_ui(fps_counter, transform, aspect_ratio);
        }

        let image_view = rdx
            .window_ctx
            .get_renderer(window_id)
            .unwrap()
            .swapchain_image_view();

        // Always call draw for proper Vulkan synchronization (even if UI is empty)
        let after_debug_ui = self.draw(image_view, last_future);
        after_debug_ui
    }
}
