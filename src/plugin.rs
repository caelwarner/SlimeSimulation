use bevy::prelude::*;
use bevy::render::{RenderApp, RenderSet};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::main_graph::node::CAMERA_DRIVER;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::AppConfig;
use crate::pipeline::{MainShaderPipeline, PipelineImages, ShaderPipelineNode};

pub struct SlimeSimulationPlugin;

impl Plugin for SlimeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationSettings>()
            .register_type::<SimulationSettings>()
            .add_plugin(ResourceInspectorPlugin::<SimulationSettings>::default())
            .add_plugin(ExtractResourcePlugin::<SimulationSettings>::default())
            .add_plugin(ExtractResourcePlugin::<PipelineImages>::default())
            .add_plugin(ExtractResourcePlugin::<PluginTime>::default())
            .add_startup_system(create_images);

        let app_config = app.world.get_resource::<AppConfig>().cloned().unwrap();
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(app_config)
            .init_resource::<SimulationSettings>()
            .init_resource::<MainShaderPipeline>()
            .add_system(queue_bind_groups.in_set(RenderSet::Queue))
            .add_system(prepare_data.in_set(RenderSet::Prepare));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(
            "simulation",
            ShaderPipelineNode::default()
        );
        render_graph.add_node_edge(
            "simulation",
            CAMERA_DRIVER,
        );
    }
}

fn create_images(mut commands: Commands, app_config: Res<AppConfig>, mut images: ResMut<Assets<Image>>) {
    let mut pipeline_images: Vec<Handle<Image>> = Vec::new();

    for _ in 0..2 {
        let mut image = Image::new_fill(
            Extent3d {
                width: app_config.texture.width,
                height: app_config.texture.height,
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
    app_config: Res<AppConfig>,
    settings: Res<SimulationSettings>,
    time: Res<PluginTime>,
) {
    pipeline.prepare_data(render_queue.as_ref(), app_config.as_ref(), settings.as_ref(), time.as_ref());
}

#[derive(Clone, ExtractResource, InspectorOptions, Reflect, Resource)]
#[reflect(InspectorOptions, Resource)]
pub struct SimulationSettings {
    pub pause: bool,
    pub num_agents: u32,
    #[inspector(min = 0.1, max = 5.0)]
    pub agent_speed: f32,
    #[inspector(min = 0.0, max = 2.0, speed = 0.05)]
    pub agent_sense_angle_offset: f32,
    #[inspector(min = 0.0, max = 30.0)]
    pub agent_sense_distance: f32,
    pub agent_turn_speed: f32,
    #[inspector(min = 0.0, max = 2.0, speed = 0.05)]
    pub agent_turn_randomness: f32,
    pub color: Color,
    pub has_trails: bool,
    #[inspector(min = 0.0, max = 5.0, speed = 0.005)]
    pub fade_rate: f32,
    #[inspector(min = 0, max = 7)]
    pub blur_radius: u32,
}

impl Default for SimulationSettings {
    fn default() -> Self {
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

#[derive(Resource)]
pub struct PluginTime {
    pub delta_time: f32,
    pub time: f32,
}

impl ExtractResource for PluginTime {
    type Source = Time;

    fn extract_resource(source: &Self::Source) -> Self {
        Self {
            delta_time: source.delta_seconds_f64() as f32,
            time: source.elapsed_seconds_f64() as f32,
        }
    }
}
