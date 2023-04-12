use viewport::{Viewport, run, RenderPassDresser};
use winit::event_loop::EventLoop;

struct MandelbrotDresser {
    render_pipeline: wgpu::RenderPipeline,
}

impl MandelbrotDresser {
    fn new(viewport: &Viewport) -> Self {
        let device = viewport.device();

        // Load shader
        let shader = unsafe {
            device
                .create_shader_module_spirv(&wgpu::include_spirv_raw!("../../target/mandelbrot.spv"))
        };

        // Create pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main_vs",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main_fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: viewport.surface_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self { render_pipeline }
    }
}

impl RenderPassDresser for MandelbrotDresser {
    fn dress<'a, 'b>(&'a self, mut render_pass: wgpu::RenderPass<'b>) where 'a: 'b {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}

pub fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let viewport = pollster::block_on(Viewport::new(800, 800, &event_loop));
    let dresser = MandelbrotDresser::new(&viewport);
    run(event_loop, viewport, dresser);
}