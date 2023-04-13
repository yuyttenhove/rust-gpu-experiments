use models::TexturedVertex;
use viewport::{RenderPassDresser, Viewport};
use wgpu::util::DeviceExt;
use winit::event_loop::EventLoop;

const VERTICES: &[TexturedVertex] = &[
    TexturedVertex {
        position: [-0.0868241, 0.49240386, 0.0],
        texture_coords: [0.4131759, 0.00759614],
    },
    TexturedVertex {
        position: [-0.49513406, 0.06958647, 0.0],
        texture_coords: [0.0048659444, 0.43041354],
    },
    TexturedVertex {
        position: [-0.21918549, -0.44939706, 0.0],
        texture_coords: [0.28081453, 0.949397],
    },
    TexturedVertex {
        position: [0.35966998, -0.3473291, 0.0],
        texture_coords: [0.85967, 0.84732914],
    },
    TexturedVertex {
        position: [0.44147372, 0.2347359, 0.0],
        texture_coords: [0.9414737, 0.2652641],
    },
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

const NUM_INDICES: u32 = INDICES.len() as u32;

struct PentagonDresser {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
}

impl PentagonDresser {
    fn new(viewport: &Viewport) -> Self {
        let device = viewport.device();
        let queue = viewport.queue();

        // Load the texture
        let diffuse_bytes = include_bytes!("../happy-tree.png"); // CHANGED!
        let diffuse_texture =
            models::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png")
                .expect("Error loading texture!");

        // Create a texture and sampler bind group
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        // Load shader
        let shader = unsafe {
            device
                .create_shader_module_spirv(&wgpu::include_spirv_raw!("../../target/textures.spv"))
        };

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main_vs",
                buffers: &[TexturedVertex::desc()],
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
            index_buffer,
            diffuse_bind_group,
        }
    }
}

impl RenderPassDresser for PentagonDresser {
    fn dress<'a, 'b>(&'a self, mut render_pass: wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..NUM_INDICES, 0, 0..1);
    }
}

pub fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let viewport = pollster::block_on(Viewport::new(1000, 1000, &event_loop));
    let dresser = PentagonDresser::new(&viewport);

    Viewport::run(viewport, event_loop, dresser);
}
