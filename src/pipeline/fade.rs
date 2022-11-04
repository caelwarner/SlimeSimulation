use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;

use crate::{AppSettings, SETTINGS};
use crate::pipeline::{get_compute_pipeline_id, SubShaderPipeline, WorkgroupSize};

pub struct FadeShaderPipeline {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: CachedComputePipelineId,
    settings: AppSettings,
}

impl FadeShaderPipeline {
    pub fn new(world: &mut World) -> Self {
        let bind_group_layout = get_bind_group_layout(
            world.resource::<RenderDevice>(),
        );

        let shader = world.resource::<AssetServer>().load("shaders/fade.wgsl");

        Self {
            pipeline: get_compute_pipeline_id(
                shader,
                world.resource_mut::<PipelineCache>().as_mut(),
                bind_group_layout.clone(),
                "fade shader update".to_string(),
                "fade".to_string(),
            ),
            bind_group_layout,
            bind_group: None,
            settings: SETTINGS,
        }
    }
}

impl SubShaderPipeline for FadeShaderPipeline {
    fn queue_bind_groups(
        &mut self,
        render_device: &RenderDevice,
        gpu_images: &RenderAssets<Image>,
        output_image: Option<&Handle<Image>>
    ) {
        self.bind_group = Some(
            render_device.create_bind_group(
                &BindGroupDescriptor {
                    label: Some("fade bind group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(
                                &gpu_images[output_image.unwrap()].texture_view,
                            ),
                        },
                    ],
                },
            ),
        )
    }

    fn get_pipeline(&self) -> CachedComputePipelineId {
        self.pipeline
    }

    fn get_bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }

    fn get_workgroup_size(&self) -> WorkgroupSize {
        WorkgroupSize {
            x: self.settings.texture_size.0 / 8,
            y: self.settings.texture_size.1 / 8,
            z: 1,
        }
    }
}

fn get_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device
        .create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("fade bind group layout"),
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
                ],
            },
        )
}
