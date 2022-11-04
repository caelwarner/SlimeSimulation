extern crate core;

use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_shader_utils::ShaderUtilsPlugin;

use crate::pipeline::PipelineOutputImage;
use crate::plugin::SlimeSimulationPlugin;

mod plugin;
mod pipeline;

const SETTINGS: AppSettings = AppSettings {
    window_size: (3440, 1440),
    texture_size: (3440, 1440),
    num_agents: 1000000,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            title: "Slime Simulation".to_string(),
            width: SETTINGS.window_size.0 as f32,
            height: SETTINGS.window_size.1 as f32,
            mode: WindowMode::SizedFullscreen,
            scale_factor_override: Some(1.0),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShaderUtilsPlugin)
        .add_plugin(SlimeSimulationPlugin)
        .add_startup_system_to_stage(
            StartupStage::PostStartup,
            setup
        )
        .run();
}

fn setup(mut commands: Commands, output_image: Res<PipelineOutputImage>) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                SETTINGS.window_size.0 as f32,
                SETTINGS.window_size.1 as f32,
            )),
            ..default()
        },
        texture: output_image.0.clone(),
        ..default()
    });

    commands.spawn_bundle(Camera2dBundle::default());
}

struct AppSettings {
    window_size: (u32, u32),
    texture_size: (u32, u32),
    num_agents: u32,
}
