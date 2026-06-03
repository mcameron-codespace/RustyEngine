pub mod bridge;
pub mod command;
pub mod context;
pub mod pipeline;

use std::sync::Arc;
use winit::window::Window;

use crate::commander::CommandBuffer;
use crate::ecs::World;

pub use bridge::{collect_renderables, MeshComponent, RenderableMesh};
pub use command::RenderCommand;
pub use context::RenderContext;
pub use pipeline::RenderPipelineContainer;

pub struct Renderer<'window> {
    pub context: RenderContext<'window>,
    pub triangle_pipeline: RenderPipelineContainer,
    pub render_queue: CommandBuffer<RenderCommand>,
}

impl<'window> Renderer<'window> {
    pub fn new(window: Arc<Window>, render_queue_capacity: usize) -> Self {
        let context = RenderContext::new(window);
        let triangle_pipeline =
            RenderPipelineContainer::new_triangle_pipeline(&context.device, context.config.format);
        let render_queue = CommandBuffer::with_capacity(render_queue_capacity);

        Self {
            context,
            triangle_pipeline,
            render_queue,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.context.resize(width, height);
    }

    pub fn collect_frame_renderables(world: &World) -> Vec<RenderableMesh> {
        collect_renderables(world)
    }

    pub fn drain_render_commands(&mut self) {
        for command in self.render_queue.drain() {
            let _ = command;
        }
    }

    pub fn render_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.drain_render_commands();

        let output = self.context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Aether Frame Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Forward Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.triangle_pipeline.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
