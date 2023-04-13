use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

use crate::RenderPassDresser;

pub struct Viewport {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    surface_format: wgpu::TextureFormat,
    window: Window,
}

impl Viewport {
    // Creating some of the wgpu types requires async code
    pub async fn new(width: u32, height: u32, event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(width, height))
            .build(event_loop)
            .unwrap();

        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.expect("Error creating surface!");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Requested adapter was none 'None'");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty() | wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .expect("Error requesting device!");

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            surface_format,
        }
    }

    pub fn run<Dresser>(mut viewport: Self, event_loop: EventLoop<()>, render_pass_dresser: Dresser)
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

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        // TODO
    }

    pub fn render<Dresser: RenderPassDresser>(
        &self,
        render_pass_dresser: &Dresser,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut renderer = Renderer::new(self)?;
        render_pass_dresser.dress(renderer.render_pass());
        renderer.render();

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

pub struct Renderer<'a> {
    encoder: wgpu::CommandEncoder,
    output: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    queue: &'a wgpu::Queue,
}

impl<'a> Renderer<'a> {
    fn new(viewport: &'a Viewport) -> Result<Self, wgpu::SurfaceError> {
        let output = viewport.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = viewport
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        Ok(Self {
            encoder,
            output,
            view,
            queue: &viewport.queue,
        })
    }

    pub fn render_pass(&mut self) -> wgpu::RenderPass {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    pub fn render(self) {
        self.queue.submit(std::iter::once(self.encoder.finish()));
        self.output.present();
    }
}
