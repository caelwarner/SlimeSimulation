use std::f32::consts::PI;
use std::num::NonZeroU32;

use bevy::core::{Pod, Zeroable};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use rand::{Rng, thread_rng};

use crate::{AppSettings, SETTINGS};
use crate::pipeline::{get_compute_pipeline_id, PipelineData, SubShaderPipeline, WorkgroupSize};

pub struct SimulationShaderPipeline {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: CachedComputePipelineId,
    settings: AppSettings,
    agents: PipelineData<Vec<Agent>>,
    context: PipelineData<SimulationPipelineContext>,
}

impl SimulationShaderPipeline {
    pub fn new(world: &mut World) -> Self {
        let bind_group_layout = get_bind_group_layout(
            world.resource::<RenderDevice>(),
            &SETTINGS,
        );

        let shader = world.resource::<AssetServer>().load("shaders/simulation.wgsl");

        Self {
            pipeline: get_compute_pipeline_id(
                shader,
                world.resource_mut::<PipelineCache>().as_mut(),
                bind_group_layout.clone(),
                "simulation shader update".to_string(),
                "update".to_string(),
            ),
            bind_group_layout,
            bind_group: None,
            settings: SETTINGS,
            agents: PipelineData::default(),
            context: PipelineData::default(),
        }
    }
}

impl SubShaderPipeline for SimulationShaderPipeline {
    fn init_data(&mut self, render_device: &RenderDevice) {
        let mut rng = thread_rng();

        self.context.data = Some(SimulationPipelineContext {
            texture_size: [
                self.settings.texture_size.0,
                self.settings.texture_size.1,
            ],
        });

        self.context.buffer = Some(render_device
            .create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("simulation settings buffer"),
                    contents: bevy::core::cast_slice(
                        &[self.context.data
                            .expect("context data to exist")]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
                }
            )
        );

        self.agents.data = Some((0..self.settings.num_agents)
            .into_iter()
            .map(|_| {
                Agent {
                    position: [
                        self.settings.texture_size.0 as f32 / 2.0,
                        self.settings.texture_size.1 as f32 / 2.0,
                    ],
                    angle: rng.gen::<f32>() * PI * 2.0,
                    _padding: 0,
                }
            }).collect::<Vec<Agent>>());

        self.agents.buffer = Some(render_device
            .create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("agents storage buffer"),
                    contents: bevy::core::cast_slice(
                        &self.agents.data
                            .as_ref()
                            .expect("agents data to exist")),
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
                            resource: self.context.buffer
                                .as_ref()
                                .expect("context buffer to exist")
                                .as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: self.agents.buffer
                                .as_ref()
                                .expect("agents buffer to exist")
                                .as_entire_binding(),
                        },
                    ],
                },
            ))
    }

    fn get_pipeline(&self) -> CachedComputePipelineId {
        self.pipeline
    }

    fn get_bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }

    fn get_workgroup_size(&self) -> WorkgroupSize {
        WorkgroupSize {
            x: self.settings.num_agents / 16,
            y: 1,
            z: 1,
        }
    }
}

fn get_bind_group_layout(render_device: &RenderDevice, settings: &AppSettings) -> BindGroupLayout {
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
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(std::mem::size_of::<SimulationPipelineContext>() as u64),
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
                            min_binding_size: BufferSize::new(std::mem::size_of::<Agent>() as u64),
                        },
                        count: Some(NonZeroU32::try_from(settings.num_agents)
                            .expect("more than zero agents")),
                    },
                ],
            },
        )
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
struct SimulationPipelineContext {
    texture_size: [u32; 2],
}


#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct Agent {
    position: [f32; 2],
    angle: f32,
    _padding: u32,
}
