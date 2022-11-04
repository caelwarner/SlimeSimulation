use std::borrow::Borrow;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice};

use crate::pipeline::fade::FadeShaderPipeline;
use crate::pipeline::simulation::SimulationShaderPipeline;

pub mod fade;
pub mod simulation;

pub struct MainShaderPipeline {
    simulation_pipeline: SimulationShaderPipeline,
    fade_pipeline: FadeShaderPipeline,
}

impl FromWorld for MainShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut pipeline = Self {
            simulation_pipeline: SimulationShaderPipeline::new(world),
            fade_pipeline: FadeShaderPipeline::new(world),
        };

        pipeline.init_data(world.resource::<RenderDevice>());
        pipeline
    }
}

impl MainShaderPipeline {
    pub fn init_data(&mut self, render_device: &RenderDevice) {
        self.simulation_pipeline.init_data(render_device);
    }

    pub fn queue_bind_groups(
        &mut self,
        render_device: Res<RenderDevice>,
        gpu_images: Res<RenderAssets<Image>>,
        output_image: Res<PipelineOutputImage>
    ) {
        self.simulation_pipeline.queue_bind_groups(
            render_device.as_ref(),
            gpu_images.as_ref(),
            Some(output_image.0.borrow()),
        );

        self.fade_pipeline.queue_bind_groups(
            render_device.as_ref(),
            gpu_images.as_ref(),
            Some(output_image.0.borrow()),
        );
    }

    fn run_shaders(&self, render_context: &mut RenderContext, world: &World) {
        let pipeline_cache = world.resource::<PipelineCache>();

        self.simulation_pipeline.run(render_context, pipeline_cache);
        self.fade_pipeline.run(render_context, pipeline_cache);
    }
}

pub trait SubShaderPipeline {
    fn init_data(&mut self, _render_device: &RenderDevice) {}

    fn queue_bind_groups(&mut self, render_device: &RenderDevice, gpu_images: &RenderAssets<Image>, output_image: Option<&Handle<Image>>);
    fn run(&self, render_context: &mut RenderContext, pipeline_cache: &PipelineCache);
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
pub struct PipelineOutputImage(pub Handle<Image>);

#[derive(Default)]
pub struct ShaderPipelineNode;

impl Node for ShaderPipelineNode {
    fn run(&self, _graph: &mut RenderGraphContext, render_context: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
        world.resource::<MainShaderPipeline>().run_shaders(render_context, world);

        Ok(())
    }
}
