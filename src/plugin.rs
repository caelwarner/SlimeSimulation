use bevy::prelude::*;
use bevy::render::{RenderApp, RenderStage};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::main_graph::node::CAMERA_DRIVER;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

use crate::pipeline::{MainShaderPipeline, PipelineImages, ShaderPipelineNode};
use crate::SETTINGS;

pub struct SlimeSimulationPlugin;

impl Plugin for SlimeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(create_images)
            .add_plugin(InspectorPlugin::<PluginSettings>::new())
            .add_plugin(ExtractResourcePlugin::<PipelineImages>::default())
            .add_plugin(ExtractResourcePlugin::<PluginSettings>::default())
            .add_plugin(ExtractResourcePlugin::<PluginTime>::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<PluginSettings>()
            .init_resource::<MainShaderPipeline>()
            .add_system_to_stage(
                RenderStage::Queue,
                queue_bind_groups,
            )
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_data,
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(
            "simulation",
            ShaderPipelineNode::default()
        );
        render_graph.add_node_edge(
            "simulation",
            CAMERA_DRIVER,
        ).unwrap();
    }
}

fn create_images(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut pipeline_images: Vec<Handle<Image>> = Vec::new();

    for _ in 0..2 {
        let mut image = Image::new_fill(
            Extent3d {
                width: SETTINGS.texture_size.0,
                height: SETTINGS.texture_size.1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8Unorm,
        );

        image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        pipeline_images.push(images.add(image));
    }

    commands.insert_resource(PipelineImages(pipeline_images));
}

fn queue_bind_groups(
    mut pipeline: ResMut<MainShaderPipeline>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<Image>>,
    output_image: Res<PipelineImages>,
) {
    pipeline.queue_bind_groups(render_device, gpu_images, output_image);
}

fn prepare_data(
    mut pipeline: ResMut<MainShaderPipeline>,
    render_queue: Res<RenderQueue>,
    settings: Res<PluginSettings>,
    time: Res<PluginTime>,
) {
    pipeline.prepare_data(render_queue, settings, time);
}

#[derive(Clone, Inspectable, ExtractResource)]
pub struct PluginSettings {
    pub pause: bool,
    pub num_agents: u32,
    #[inspectable(min = 0.1, max = 5.0)]
    pub agent_speed: f32,
    #[inspectable(min = 0.0, max = 2.0, speed = 0.05)]
    pub agent_sense_angle_offset: f32,
    #[inspectable(min = 0.0, max = 30.0)]
    pub agent_sense_distance: f32,
    pub agent_turn_speed: f32,
    #[inspectable(min = 0.0, max = 2.0, speed = 0.05)]
    pub agent_turn_randomness: f32,
    pub color: Color,
    pub has_trails: bool,
    #[inspectable(min = 0.0, max = 5.0, speed = 0.005)]
    pub fade_rate: f32,
    #[inspectable(min = 0, max = 7)]
    pub blur_radius: u32,
}

impl FromWorld for PluginSettings {
    fn from_world(_world: &mut World) -> Self {
        Self {
            pause: true,
            num_agents: 1000000,
            agent_speed: 1.0,
            agent_sense_angle_offset: 0.5,
            agent_sense_distance: 20.0,
            agent_turn_speed: 1.0,
            agent_turn_randomness: 0.1,
            color: Color::WHITE,
            has_trails: true,
            fade_rate: 0.15,
            blur_radius: 1,
        }
    }
}

pub struct PluginTime {
    pub delta_time: f32,
    pub time: f32,
}

impl ExtractResource for PluginTime {
    type Source = Time;

    fn extract_resource(source: &Self::Source) -> Self {
        Self {
            delta_time: source.delta_seconds_f64() as f32,
            time: source.seconds_since_startup() as f32,
        }
    }
}
