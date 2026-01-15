use std::sync::Arc;

use egui_winit_vulkano::{Gui, egui};
use vulkano::{image::view::ImageView, sync::GpuFuture};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use crate::{counter::FpsCounter, transform::TransformState};

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
                ..Default::default()
            },
        );

        Self { gui }
    }

    pub fn update(&mut self, event: &WindowEvent) -> bool {
        self.gui.update(event)
    }

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
                .resizable(true)
                .default_width(1000.0)
                .show(&ctx, |ui| {
                    ui.label(format!("FPS: {:.1}", fps_counter.fps()));
                    ui.label(format!("Frame Time: {:.2} ms", fps_counter.frame_time_ms()));

                    ui.separator();
                    ui.heading("Transform State");
                    ui.label(format!("{:#?}", transform));

                    ui.separator();
                    ui.heading("Vertices (Original)");
                    egui::ScrollArea::vertical()
                        .id_salt("original_vertices")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.label(format!("{:#?}", crate::models::POSITIONS));
                        });

                    ui.separator();
                    ui.heading("Vertices (Transformed)");
                    egui::ScrollArea::vertical()
                        .id_salt("transformed_vertices")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (i, vertex) in crate::models::POSITIONS.iter().enumerate() {
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
        });
    }

    pub fn draw(
        &mut self,
        image_view: Arc<ImageView>,
        last_future: Box<dyn GpuFuture>,
    ) -> Box<dyn GpuFuture> {
        self.gui.draw_on_image(last_future, image_view)
    }
}
