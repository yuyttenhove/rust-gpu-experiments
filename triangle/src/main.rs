use models::ColoredVertex;
use viewport::{RenderPassDresser, Viewport};
use wgpu::util::DeviceExt;
use winit::event_loop::EventLoop;

const VERTICES: &[ColoredVertex] = &[
    ColoredVertex {
        position: [0.0, 0.36602540378, 0.0],
        color: [0.5, 1., 0.],
    },
    ColoredVertex {
        position: [-0.5, -0.5, 0.0],
        color: [0., 0., 1.],
    },
    ColoredVertex {
        position: [0.5, -0.5, 0.0],
        color: [1., 0., 0.],
    },
];

const NUM_VERTICES: u32 = VERTICES.len() as u32;

struct TriangleDresser {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl TriangleDresser {
    fn new(viewport: &Viewport) -> Self {
        let device = viewport.device();

        // Load shader
        let shader = unsafe {
            device
                .create_shader_module_spirv(&wgpu::include_spirv_raw!("../../target/triangle.spv"))
        };

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

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
                buffers: &[ColoredVertex::desc()],
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
                front_face: wgpu::FrontFace::Ccw,
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

        Self {
            render_pipeline,
            vertex_buffer,
        }
    }
}

impl RenderPassDresser for TriangleDresser {
    fn dress<'a, 'b>(&'a self, mut render_pass: wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..NUM_VERTICES, 0..1);
    }
}

pub fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let viewport = pollster::block_on(Viewport::new(1000, 1000, &event_loop));
    let dresser = TriangleDresser::new(&viewport);

    Viewport::run(viewport, event_loop, dresser);
}
