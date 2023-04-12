pub trait RenderPassDresser {
    fn dress<'a, 'b>(&'a self, render_pass: wgpu::RenderPass<'b>) where 'a: 'b;
}