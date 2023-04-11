mod state;
mod vertex;

use state::State;
use vertex::Vertex;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.36602540378, 0.0], color: [0., 1., 0.],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0], color: [0., 0., 1.],
    },
    Vertex {
        position: [0.5, -0.5, 0.0], color: [1., 0., 0.],
    },
];

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(800, 800))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => match event {
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                // new_inner_size is &&mut so we have to dereference it twice
                state.resize(**new_inner_size);
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
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        _ => {}
    });
}

pub fn main() {
    pollster::block_on(run());
}
