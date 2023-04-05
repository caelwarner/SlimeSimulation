use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};

use crate::pipeline::blur::BlurShaderPipeline;
use crate::pipeline::fade::FadeShaderPipeline;
use crate::pipeline::recolor::RecolorShaderPipeline;
use crate::pipeline::simulation::SimulationShaderPipeline;
use crate::plugin::{PluginSettings, PluginTime};

pub mod blur;
pub mod fade;
pub mod recolor;
pub mod simulation;

pub struct MainShaderPipeline {
    sub_pipelines: Vec<Box<dyn SubShaderPipeline>>,
}

impl FromWorld for MainShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut pipeline = Self {
            sub_pipelines: vec![
                Box::new(SimulationShaderPipeline::new(world)),
                Box::new(FadeShaderPipeline::new(world)),
                Box::new(BlurShaderPipeline::new(world)),
                Box::new(RecolorShaderPipeline::new(world)),
            ],
        };

        pipeline.init_data(world.resource::<RenderDevice>(), world.resource::<PluginSettings>());
        pipeline
    }
}

impl MainShaderPipeline {
    pub fn init_data(&mut self, render_device: &RenderDevice, settings: &PluginSettings) {
        for sub_pipeline in &mut self.sub_pipelines {
            sub_pipeline.init_data(render_device, settings);
        }
    }

    pub fn prepare_data(&mut self, render_queue: Res<RenderQueue>, settings: Res<PluginSettings>, time: Res<PluginTime>) {
        for sub_pipeline in &mut self.sub_pipelines {
            sub_pipeline.prepare_data(render_queue.as_ref(), settings.as_ref(), time.as_ref());
        }
    }

    pub fn queue_bind_groups(
        &mut self,
        render_device: Res<RenderDevice>,
        gpu_images: Res<RenderAssets<Image>>,
        images: Res<PipelineImages>,
    ) {
        for sub_pipeline in &mut self.sub_pipelines {
            sub_pipeline.queue_bind_groups(
                render_device.as_ref(),
                gpu_images.as_ref(),
                images.0.as_ref(),
            )
        }
    }

    fn run_shaders(&self, render_context: &mut RenderContext, world: &World) {
        let pipeline_cache = world.resource::<PipelineCache>();

        for sub_pipeline in &self.sub_pipelines {
            run_shader(
                render_context,
                pipeline_cache,
                sub_pipeline.get_pipeline(),
                sub_pipeline.get_bind_group(),
                sub_pipeline.get_workgroup_size(world.resource::<PluginSettings>()),
            )
        }
    }
}

fn run_shader(
    render_context: &mut RenderContext,
    pipeline_cache: &PipelineCache,
    pipeline: CachedComputePipelineId,
    bind_group: Option<&BindGroup>,
    workgroup_size: WorkgroupSize,
) {
    if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline) {
        let mut compute_pass = render_context.command_encoder
            .begin_compute_pass(&ComputePassDescriptor::default());

        compute_pass.set_bind_group(0, bind_group.expect("bind group to exist"), &[]);

        let pipeline = pipeline_cache
            .get_compute_pipeline(pipeline)
            .expect("pipeline to exist in pipeline cache");

        compute_pass.set_pipeline(pipeline);
        compute_pass.dispatch_workgroups(
            workgroup_size.x,
            workgroup_size.y,
            workgroup_size.z,
        );
    }
}

fn get_compute_pipeline_id(
    shader: Handle<Shader>,
    pipeline_cache: &mut PipelineCache,
    bind_group_layout: BindGroupLayout,
    label: String,
    entry_point: String,
) -> CachedComputePipelineId {
    pipeline_cache
        .queue_compute_pipeline(
            ComputePipelineDescriptor {
                label: Some(Cow::from(label)),
                layout: Some(vec![bind_group_layout]),
                shader,
                shader_defs: vec![],
                entry_point: Cow::from(entry_point),
            },
        )
}

pub trait SubShaderPipeline: Send + Sync {
    fn init_data(&mut self, _render_device: &RenderDevice, _settings: &PluginSettings) {}
    fn prepare_data(&mut self, _render_queue: &RenderQueue, _settings: &PluginSettings, _time: &PluginTime) {}

    fn queue_bind_groups(&mut self, render_device: &RenderDevice, gpu_images: &RenderAssets<Image>, images: &Vec<Handle<Image>>);
    fn get_pipeline(&self) -> CachedComputePipelineId;
    fn get_bind_group(&self) -> Option<&BindGroup>;
    fn get_workgroup_size(&self, settings: &PluginSettings) -> WorkgroupSize;
}

pub struct PipelineData<T> {
    data: Option<T>,
    buffer: Option<Buffer>,
}

impl<T> Default for PipelineData<T> {
    fn default() -> Self {
        Self {
            data: None,
            buffer: None,
        }
    }
}

#[derive(Clone, Deref, ExtractResource)]
pub struct PipelineImages(pub Vec<Handle<Image>>);

#[derive(Default)]
pub struct ShaderPipelineNode;

impl Node for ShaderPipelineNode {
    fn run(&self, _graph: &mut RenderGraphContext, render_context: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
        world.resource::<MainShaderPipeline>().run_shaders(render_context, world);

        Ok(())
    }
}

pub struct WorkgroupSize {
    x: u32,
    y: u32,
    z: u32,
}