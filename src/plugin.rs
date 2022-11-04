use bevy::prelude::*;
use bevy::render::{RenderApp, RenderStage};
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::renderer::RenderDevice;

use crate::pipeline::{MainShaderPipeline, PipelineOutputImage, ShaderPipelineNode};
use crate::pipeline::simulation::SimulationPipelineContext;
use crate::SETTINGS;

pub struct SlimeSimulationPlugin;

impl Plugin for SlimeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(create_output_image)
            .add_plugin(ExtractResourcePlugin::<PipelineOutputImage>::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(SimulationPipelineContext {
                num_agents: SETTINGS.num_agents,
                window_size: SETTINGS.window_size,
            })
            .init_resource::<MainShaderPipeline>()
            .add_system_to_stage(
                RenderStage::Queue,
                queue_bind_groups,
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(
            "simulation",
            ShaderPipelineNode::default()
        );
        render_graph.add_node_edge(
            "simulation",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        ).unwrap();
    }
}

fn create_output_image(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SETTINGS.window_size.0,
            height: SETTINGS.window_size.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    image.texture_descriptor.usage = TextureUsages::COPY_DST
        | TextureUsages::STORAGE_BINDING
        | TextureUsages::TEXTURE_BINDING;

    commands.insert_resource(PipelineOutputImage(images.add(image)));
}

fn queue_bind_groups(
    mut pipeline: ResMut<MainShaderPipeline>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<Image>>,
    output_image: Res<PipelineOutputImage>,
) {
    pipeline.queue_bind_groups(render_device, gpu_images, output_image);
}
