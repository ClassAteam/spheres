use glam::Vec3;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::context::VulkanBasicContext;
use crate::control::TransformState;
use crate::render::RenderContext;

pub struct FpsCounter {
    last_frame: Instant,
    frame_times: Vec<f32>,
    max_samples: usize,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_times: Vec::with_capacity(60),
            max_samples: 60,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.frame_times.push(delta);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
    }

    pub fn fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg_time > 0.0 {
            1.0 / avg_time
        } else {
            0.0
        }
    }

    pub fn frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let avg_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        avg_time * 1000.0
    }
}

pub struct App {
    pub basic_context: Arc<VulkanBasicContext>,
    pub rdx: Option<RenderContext>,
    transform: TransformState,
    fps_counter: FpsCounter,
}

impl App {
    pub fn new() -> Self {
        let context = VulkanBasicContext::new();
        App {
            basic_context: Arc::new(context),
            transform: TransformState::new(),
            rdx: None,
            fps_counter: FpsCounter::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rdx = Some(RenderContext::new(
            event_loop,
            self.basic_context.bctx.clone(),
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui handle the event first
        if let Some(rdx) = &mut self.rdx {
            let consumed = rdx.gui.update(&event);

            // If egui consumed the event (user is interacting with GUI), don't process it further
            if consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.fps_counter.update();
                self.rdx.as_mut().unwrap().draw(
                    self.basic_context.cb_alloc.clone(),
                    self.basic_context.bctx.memory_allocator().clone(),
                    self.basic_context.descriptor_set_allocator.clone(),
                    &self.transform,
                    &self.fps_counter,
                );
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key_code {
                KeyCode::KeyL => self.transform.rotate_model(Vec3::new(0.0, -0.01, 0.0)),
                KeyCode::KeyH => self.transform.rotate_model(Vec3::new(0.0, 0.01, 0.0)),
                KeyCode::KeyJ => self.transform.rotate_model(Vec3::new(-0.01, 0.0, 0.0)),
                KeyCode::KeyK => self.transform.rotate_model(Vec3::new(0.01, 0.0, 0.0)),
                KeyCode::Escape => event_loop.exit(),
                _ => (),
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw for continuous rendering
        if let Some(rdx) = &self.rdx {
            rdx.window_ctx
                .get_window(rdx.id)
                .expect("Failed to get window")
                .request_redraw();
        }
    }
}
