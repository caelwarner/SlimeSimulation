use std::borrow::Cow;
use std::f32::consts::PI;
use std::num::NonZeroU32;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice};
use bytemuck::{Pod, Zeroable};
use rand::{Rng, thread_rng};

use crate::pipeline::{PipelineData, SubShaderPipeline};

const WORKGROUP_SIZE: u32 = 16;

pub struct SimulationShaderPipeline {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: CachedComputePipelineId,
    context: SimulationPipelineContext,
    agents: PipelineData<Vec<Agent>>,
}

impl SimulationShaderPipeline {
    pub fn new(world: &mut World) -> Self {
        let context = world
            .remove_resource::<SimulationPipelineContext>()
            .unwrap();

        let bind_group_layout = get_bind_group_layout(
            world.resource::<RenderDevice>(),
            &context,
        );

        let shader = world.resource::<AssetServer>().load("shaders/simulation.wgsl");

        Self {
            pipeline: get_compute_pipeline_id(
                shader,
                world.resource_mut::<PipelineCache>().as_mut(),
                bind_group_layout.clone()
            ),
            bind_group_layout,
            bind_group: None,
            context,
            agents: PipelineData::default(),
        }
    }
}

impl SubShaderPipeline for SimulationShaderPipeline {
    fn init_data(&mut self, render_device: &RenderDevice) {
        let mut rng = thread_rng();

        self.agents.data = Some((0..self.context.num_agents)
            .into_iter()
            .map(|_| {
                Agent {
                    position: [
                        self.context.texture_size.0 as f32 / 2.0,
                        self.context.texture_size.1 as f32 / 2.0,
                    ],
                    angle: rng.gen::<f32>() * PI * 2.0,
                    _padding: 0,
                }
            }).collect::<Vec<Agent>>());

        self.agents.buffer = Some(render_device
            .create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("agents storage buffer"),
                    contents: bytemuck::cast_slice(&self.agents.data.as_ref().expect("agents buffer to exist")),
                    usage: BufferUsages::STORAGE,
                }
            ));
    }

    fn queue_bind_groups(
        &mut self,
        render_device: &RenderDevice,
        gpu_images: &RenderAssets<Image>,
        output_image: Option<&Handle<Image>>,
    ) {
        self.bind_group = Some(
            render_device.create_bind_group(
                &BindGroupDescriptor {
                    label: Some("simulation bind group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(
                                &gpu_images[output_image.unwrap()].texture_view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: self.agents.buffer.as_ref()
                                .expect("agents buffer to exist")
                                .as_entire_binding(),
                        },
                    ],
                },
            ))
    }

    fn run(&self, render_context: &mut RenderContext, pipeline_cache: &PipelineCache) {
        let mut compute_pass = render_context.command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        compute_pass.set_bind_group(0, self.bind_group.as_ref().expect("bind group to exist"), &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(self.pipeline)
            .expect("pipeline to exist in pipeline cache");

        compute_pass.set_pipeline(pipeline);
        compute_pass.dispatch_workgroups(
            self.context.num_agents / WORKGROUP_SIZE,
            1,
            1,
        );
    }
}

fn get_bind_group_layout(render_device: &RenderDevice, context: &SimulationPipelineContext) -> BindGroupLayout {
    render_device
        .create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("simulation bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage {
                                read_only: false,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(std::mem::size_of::<Agent>() as u64),
                        },
                        count: Some(NonZeroU32::try_from(context.num_agents)
                            .expect("more than zero agents")),
                    },
                ],
            },
        )
}

fn get_compute_pipeline_id(
    shader: Handle<Shader>,
    pipeline_cache: &mut PipelineCache,
    bind_group_layout: BindGroupLayout,
) -> CachedComputePipelineId {
    pipeline_cache
        .queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some(Cow::from("simulation shader update")),
                layout: Some(vec![bind_group_layout]),
                shader,
                shader_defs: vec![],
                entry_point: Cow::from("update"),
            },
        )
}

#[derive(Clone)]
pub struct SimulationPipelineContext {
    pub num_agents: u32,
    pub texture_size: (u32, u32),
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct Agent {
    position: [f32; 2],
    angle: f32,
    _padding: u32,
}


