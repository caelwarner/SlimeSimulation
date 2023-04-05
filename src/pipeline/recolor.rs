use bevy::asset::Handle;
use bevy::core::{Pod, Zeroable};
use bevy::prelude::{AssetServer, Image, World};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};

use crate::pipeline::{get_compute_pipeline_id, PipelineData, SubShaderPipeline, WorkgroupSize};
use crate::plugin::{PluginSettings, PluginTime};
use crate::SETTINGS;

pub struct RecolorShaderPipeline {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: CachedComputePipelineId,
    context: PipelineData<RecolorPipelineContext>,
}

impl RecolorShaderPipeline {
    pub fn new(world: &mut World) -> Self {
        let bind_group_layout = get_bind_group_layout(
            world.resource::<RenderDevice>(),
        );

        let shader = world.resource::<AssetServer>().load("shaders/recolor.wgsl");

        Self {
            pipeline: get_compute_pipeline_id(
                shader,
                world.resource_mut::<PipelineCache>().as_mut(),
                bind_group_layout.clone(),
                "recolor shader update".to_string(),
                "recolor".to_string(),
            ),
            bind_group_layout,
            bind_group: None,
            context: PipelineData::default(),
        }
    }
}

impl SubShaderPipeline for RecolorShaderPipeline {
    fn init_data(&mut self, render_device: &RenderDevice, _settings: &PluginSettings) {
        self.context.buffer = Some(render_device
            .create_buffer(
                &BufferDescriptor {
                    label: Some("recolor context uniform buffer"),
                    size: std::mem::size_of::<RecolorPipelineContext>() as u64,
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            )
        );
    }

    fn prepare_data(&mut self, render_queue: &RenderQueue, settings: &PluginSettings, _time: &PluginTime) {
        self.context.data = Some(RecolorPipelineContext {
            color: settings.color.as_rgba_f32(),
        });

        render_queue.write_buffer(
            self.context.buffer.as_ref().expect("context buffer to exist"),
            0,
            bevy::core::cast_slice(&[
                self.context.data.expect("context data to exist"),
            ]),
        );
    }

    fn queue_bind_groups(&mut self, render_device: &RenderDevice, gpu_images: &RenderAssets<Image>, images: &Vec<Handle<Image>>) {
        self.bind_group = Some(
            render_device.create_bind_group(
                &BindGroupDescriptor {
                    label: Some("recolor bind group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(
                                &gpu_images[images.get(1).unwrap()].texture_view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: self.context.buffer
                                .as_ref()
                                .expect("context buffer to exist")
                                .as_entire_binding(),
                        },
                    ],
                },
            )
        )
    }

    fn get_pipeline(&self) -> CachedComputePipelineId {
        self.pipeline
    }

    fn get_bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }

    fn get_workgroup_size(&self, _settings: &PluginSettings) -> WorkgroupSize {
        WorkgroupSize {
            x: SETTINGS.texture_size.0 / 8,
            y: SETTINGS.texture_size.1 / 8,
            z: 1,
        }
    }
}

fn get_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device
        .create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("recolor bind group layout"),
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
                            min_binding_size: BufferSize::new(std::mem::size_of::<RecolorPipelineContext>() as u64),
                        },
                        count: None,
                    },
                ],
            },
        )
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
struct RecolorPipelineContext {
    color: [f32; 4],
}
