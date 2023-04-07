use bevy::core::{Pod, Zeroable};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};

use crate::AppConfig;
use crate::pipeline::{get_compute_pipeline_id, PipelineData, SubShaderPipeline};
use crate::plugin::{PluginTime, SimulationSettings};

pub struct BlurShaderPipeline {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: CachedComputePipelineId,
    context: PipelineData<BlurPipelineContext>,
}

impl BlurShaderPipeline {
    pub fn new(world: &mut World) -> Self {
        let bind_group_layout = get_bind_group_layout(
            world.resource::<RenderDevice>(),
        );

        let shader = world.resource::<AssetServer>().load("shaders/blur.wgsl");

        Self {
            pipeline: get_compute_pipeline_id(
                shader,
                world.resource_mut::<PipelineCache>().as_mut(),
                bind_group_layout.clone(),
                "blur shader update".to_string(),
                "blur".to_string(),
            ),
            bind_group_layout,
            bind_group: None,
            context: PipelineData::default(),
        }
    }
}

impl SubShaderPipeline for BlurShaderPipeline {
    fn init_data(&mut self, render_device: &RenderDevice, _app_config: &AppConfig, _settings: &SimulationSettings) {
        self.context.buffer = Some(render_device
           .create_buffer(
               &BufferDescriptor {
                   label: Some("blur context uniform buffer"),
                   size: std::mem::size_of::<BlurPipelineContext>() as u64,
                   usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                   mapped_at_creation: false,
               },
           ),
        );
    }

    fn prepare_data(&mut self, render_queue: &RenderQueue, app_config: &AppConfig, settings: &SimulationSettings, _time: &PluginTime) {
        self.context.data = Some(BlurPipelineContext {
            pause: if settings.pause { 1 } else { 0 },
            width: app_config.texture.width,
            height: app_config.texture.height,
            blur_radius: settings.blur_radius,
        });

        render_queue.write_buffer(
            self.context.buffer.as_ref().expect("context buffer to exist"),
            0,
            bevy::core::cast_slice(&[
                self.context.data.expect("context data to exist"),
            ]),
        );
    }

    fn queue_bind_groups(
        &mut self,
        render_device: &RenderDevice,
        gpu_images: &RenderAssets<Image>,
        images: &Vec<Handle<Image>>,
    ) {
        self.bind_group = Some(
            render_device.create_bind_group(
                &BindGroupDescriptor {
                    label: Some("blur bind group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(
                                &gpu_images[images.get(0).unwrap()].texture_view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(
                                &gpu_images[images.get(1).unwrap()].texture_view,
                            ),
                        },
                        BindGroupEntry {
                            binding: 2,
                            resource: self.context.buffer
                                .as_ref()
                                .expect("context buffer to exist")
                                .as_entire_binding(),
                        },
                    ],
                },
            ),
        );
    }

    fn get_pipeline(&self) -> CachedComputePipelineId {
        self.pipeline
    }

    fn get_bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }
}

fn get_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device
        .create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("blur bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(std::mem::size_of::<BlurPipelineContext>() as u64),
                        },
                        count: None,
                    },
                ],
            },
        )
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
struct BlurPipelineContext {
    pause: u32,
    width: u32,
    height: u32,
    blur_radius: u32,
}
