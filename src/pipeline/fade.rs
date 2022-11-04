use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages, StorageTextureAccess, TextureFormat, TextureViewDimension};
use bevy::render::renderer::{RenderContext, RenderDevice};

use crate::pipeline::SubShaderPipeline;

const WORKGROUP_SIZE: u32 = 8;

pub struct FadeShaderPipeline {
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    pipeline: CachedComputePipelineId,
    context: FadePipelineContext,
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
            ),
            bind_group_layout,
            bind_group: None,
            context: world
                .remove_resource::<FadePipelineContext>()
                .unwrap(),
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

    fn run(&self, render_context: &mut RenderContext, pipeline_cache: &PipelineCache) {
        let mut compute_pass = render_context.command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        compute_pass.set_bind_group(0, self.bind_group.as_ref().expect("bind group to exist"), &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(self.pipeline)
            .expect("pipeline to exist in pipeline cache");

        compute_pass.set_pipeline(pipeline);
        compute_pass.dispatch_workgroups(
            self.context.texture_size.0 / WORKGROUP_SIZE,
            self.context.texture_size.1 / WORKGROUP_SIZE,
            1,
        );
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

fn get_compute_pipeline_id(
    shader: Handle<Shader>,
    pipeline_cache: &mut PipelineCache,
    bind_group_layout: BindGroupLayout,
) -> CachedComputePipelineId {
    pipeline_cache
        .queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some(Cow::from("fade shader update")),
                layout: Some(vec![bind_group_layout]),
                shader,
                shader_defs: vec![],
                entry_point: Cow::from("fade"),
            },
        )
}

pub struct FadePipelineContext {
    pub texture_size: (u32, u32),
}
