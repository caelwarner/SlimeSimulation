use std::borrow::Cow;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::{render_graph, RenderApp, RenderStage};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{NodeRunError, RenderGraph, RenderGraphContext};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};

use bevy_shader_utils::ShaderUtilsPlugin;
use bytemuck::{Pod, Zeroable};
use rand::{Rng, thread_rng};

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShaderUtilsPlugin)
        .add_plugin(SlimeSimulationPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    image.texture_descriptor.usage = TextureUsages::COPY_DST
        | TextureUsages::STORAGE_BINDING
        | TextureUsages::TEXTURE_BINDING;

    let image = images.add(image);

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                SIZE.0 as f32,
                SIZE.1 as f32,
            )),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn_bundle(Camera2dBundle::default());

    commands.insert_resource(SlimeSimulationImage(image));
}

pub struct SlimeSimulationPlugin;

impl Plugin for SlimeSimulationPlugin {
    fn build(&self, app: &mut App) {
        let render_device = app.world.resource::<RenderDevice>();
        let mut rng = thread_rng();

        let agents = (0..128)
            .into_iter()
            .map(|_| {
                Agent {
                    position: [SIZE.0 as f32 / 2.0, SIZE.1 as f32 / 2.0],
                    angle: rng.gen::<f32>() * PI * 2.0,
                    _padding: 0,
                }
            })
            .collect::<Vec<Agent>>();

        let time_buffer = render_device.create_buffer(
            &BufferDescriptor {
                label: Some("time uniform buffer"),
                size: std::mem::size_of::<f32>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        let agents_buffer = render_device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: Some("agents storage buffer"),
                contents: bytemuck::cast_slice(&agents),
                usage: BufferUsages::STORAGE,
            },
        );

        app.add_plugin(ExtractResourcePlugin::<SlimeSimulationImage>::default())
            .add_plugin(ExtractResourcePlugin::<ExtractedTime>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<SlimeSimulationPipeline>()
            .insert_resource(TimeMeta {
                buffer: time_buffer,
                bind_group: None,
            })
            .insert_resource(ExtractedAgents {
                agents
            })
            .insert_resource(AgentsMeta {
                buffer: agents_buffer,
                bind_group: None,
            })
            .add_system_to_stage(
                RenderStage::Queue,
                queue_bind_group,
            )
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_time,
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("slime_simulation", SlimeSimulationNode::default());
        render_graph.add_node_edge(
            "slime_simulation",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        ).unwrap();
    }
}

#[derive(Clone, Deref, ExtractResource)]
struct SlimeSimulationImage(Handle<Image>);

struct SlimeSimulationBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<SlimeSimulationPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    slime_simulation_image: Res<SlimeSimulationImage>,
    render_device: Res<RenderDevice>,
    time_meta: ResMut<TimeMeta>,
    agents_meta: ResMut<AgentsMeta>,
) {
    let view = &gpu_images[&slime_simulation_image.0];
    let bind_group = render_device.create_bind_group(
        &BindGroupDescriptor {
            label: None,
            layout: &pipeline.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: time_meta.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: agents_meta.buffer.as_entire_binding(),
                },
            ],
        },
    );

    commands.insert_resource(SlimeSimulationBindGroup(bind_group));
}

pub struct SlimeSimulationPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for SlimeSimulationPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout = world
            .resource::<RenderDevice>()
            .create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba8Unorm,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(std::mem::size_of::<f32>() as u64),
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Storage {
                                    read_only: false,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(std::mem::size_of::<[Agent; 2]>() as u64),
                            },
                            count: None,
                        },
                    ],
                }
            );

        let shader = world.resource::<AssetServer>().load("shaders/flow.wgsl");
        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let init_pipeline = pipeline_cache
            .queue_compute_pipeline(
                ComputePipelineDescriptor {
                    label: None,
                    layout: Some(vec![texture_bind_group_layout.clone()]),
                    shader: shader.clone(),
                    shader_defs: vec![],
                    entry_point: Cow::from("init"),
                },
            );

        let update_pipeline = pipeline_cache
            .queue_compute_pipeline(
                ComputePipelineDescriptor {
                    label: None,
                    layout: Some(vec![texture_bind_group_layout.clone()]),
                    shader,
                    shader_defs: vec![],
                    entry_point: Cow::from("update"),
                }
            );

        SlimeSimulationPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

enum SlimeSimulationState {
    Loading,
    Init,
    Update,
}

struct SlimeSimulationNode {
    state: SlimeSimulationState,
}

impl Default for SlimeSimulationNode {
    fn default() -> Self {
        Self {
            state: SlimeSimulationState::Loading,
        }
    }
}

impl render_graph::Node for SlimeSimulationNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<SlimeSimulationPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            SlimeSimulationState::Loading => {
                if let CachedPipelineState::Ok(_) = pipeline_cache
                    .get_compute_pipeline_state(pipeline.init_pipeline) {
                    self.state = SlimeSimulationState::Init;
                }
            },
            SlimeSimulationState::Init => {
                if let CachedPipelineState::Ok(_) = pipeline_cache
                    .get_compute_pipeline_state(pipeline.update_pipeline) {
                    self.state = SlimeSimulationState::Update;
                }
            },
            SlimeSimulationState::Update => {}
        }
    }

    fn run(&self, _graph: &mut RenderGraphContext, render_context: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
        let texture_bind_group = &world.resource::<SlimeSimulationBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<SlimeSimulationPipeline>();

        let mut pass = render_context.command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        match self.state {
            SlimeSimulationState::Loading => {},
            SlimeSimulationState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();

                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    128 / 16,
                    1,
                    1,
                );
            },
            SlimeSimulationState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();

                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(
                    128 / 16,
                    1,
                    1,
                );
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct ExtractedTime {
    seconds_since_startup: f32,
}

impl ExtractResource for ExtractedTime {
    type Source = Time;

    fn extract_resource(time: &Self::Source) -> Self {
        Self {
            seconds_since_startup: time.seconds_since_startup() as f32,
        }
    }
}

struct TimeMeta {
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}

fn prepare_time(time: Res<ExtractedTime>, time_meta: ResMut<TimeMeta>, render_queue: Res<RenderQueue>) {
    render_queue.write_buffer(
        &time_meta.buffer,
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
struct Agent {
    position: [f32; 2],
    angle: f32,
    _padding: u32,
}

#[derive(Default)]
struct ExtractedAgents {
    agents: Vec<Agent>,
}

struct AgentsMeta {
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}
