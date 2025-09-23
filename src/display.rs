use crate::ctx::Ctx;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

#[derive(Default)]
struct App<'window> {
    window: Option<Arc<Window>>,
    ctx: Option<Ctx<'window>>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("winit exemple"))
                .expect("Unable to create window"),
        );
        let ctx = Ctx::new(window.clone());
        self.window = Some(window.clone());
        self.ctx = Some(ctx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(ctx), Some(window)) = (self.ctx.as_mut(), self.window.as_ref()) {
                    println!("Resize {:?}", new_size);
                    ctx.resize(new_size);
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        repeat: false,
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Enter),
                        ..
                    },
                ..
            } => {
                println!("Press space {:?}", event);
            }
            WindowEvent::RedrawRequested => {
                println!("Redraw");
                if let Some(ctx) = self.ctx.as_mut() {
                    ctx.draw().expect("Unable to draw");
                }
            }
            _ => (),
        }
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Unable to run app");
}
