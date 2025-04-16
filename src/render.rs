use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    width: u32,
    height: u32,
    video_memory: Vec<u8>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_position(LogicalPosition::new(0, 0))
            .with_title("Virtual Machine Window");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let surface_texture = SurfaceTexture::new(self.width, self.height, window.clone());
        let pixels = Pixels::new(self.width, self.height, surface_texture).unwrap();

        self.pixels = Some(pixels);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.render(self.video_memory.clone());
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

impl App {
    fn render(&mut self, video_memory: Vec<u8>) {
        if let Some(pixels) = self.pixels.as_mut() {
            let frame = pixels.frame_mut();

            if video_memory.len() < frame.len() {
                eprintln!(
                    "Error: Video memory size does not match framebuffer size. Frame size: {}",
                    frame.len()
                );
                return;
            }

            frame.copy_from_slice(&video_memory);
            pixels.render().unwrap();
        }
    }

    pub fn new(width: u32, height: u32, video_memory: Vec<u8>) -> Self {
        App {
            window: None,
            pixels: None,
            width,
            height,
            video_memory,
        }
    }
}
