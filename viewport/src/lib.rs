mod viewport;
mod render_pass_dresser;

pub use viewport::Viewport;
pub use render_pass_dresser::RenderPassDresser;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run<Dresser>(event_loop: EventLoop<()>, mut viewport: Viewport, render_pass_dresser: Dresser)
where
    Dresser: RenderPassDresser + 'static,
{
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == viewport.window().id() => match event {
            WindowEvent::Resized(physical_size) => {
                viewport.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                viewport.resize(**new_inner_size);
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == viewport.window().id() => {
            viewport.update();
            match viewport.render(&render_pass_dresser) {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => viewport.resize(viewport.size()),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            viewport.window().request_redraw();
        }
        _ => {}
    });
}
