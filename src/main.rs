#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;

use bevy::asset::AssetPlugin;
use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode, WindowResized, WindowResolution};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use serde::{Deserialize, Serialize};

use crate::pipeline::PipelineImages;
use crate::plugin::SlimeSimulationPlugin;

mod plugin;
mod pipeline;

const CONFIG_FILE_NAME: &str = "slime_simulation_config.toml";

fn main() {
    let config: AppConfig = match fs::read_to_string(CONFIG_FILE_NAME) {
        Ok(contents) => {
            toml::from_str(contents.as_str()).unwrap()
        },
        Err(..) => {
            fs::write(CONFIG_FILE_NAME, toml::to_string(&AppConfig::default()).unwrap()).unwrap();
            AppConfig::default()
        },
    };

    let mut window_resolution = WindowResolution::new(
        config.window.width as f32,
        config.window.height as f32,
    );
    window_resolution.set_scale_factor_override(if config.window.override_scale_factor { Some(1.0) } else { None });

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(config.clone())
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Slime Simulation"),
                    resolution: window_resolution,
                    resizable: config.window.resizable,
                    mode: if config.window.fullscreen { WindowMode::Fullscreen } else { WindowMode::Windowed },
                    present_mode: if config.window.vsync { PresentMode::AutoVsync } else { PresentMode::AutoNoVsync },
                    ..default()
                }),
                ..default()
            })
            .build()
            .add_before::<AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(SlimeSimulationPlugin)
        .add_startup_system(setup.in_base_set(StartupSet::PostStartup))
        .add_system(on_window_resize)
        .run();
}

fn setup(mut commands: Commands, images: Res<PipelineImages>, config: Res<AppConfig>) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(
                config.window.width as f32,
                config.window.height as f32,
            )),
            ..default()
        },
        texture: images.0.last().cloned().unwrap(),
        ..default()
    });

    commands.spawn(Camera2dBundle::default());
}

fn on_window_resize(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Sprite>,
) {
    for event in resize_events.iter() {
        let mut sprite = query.single_mut();
        sprite.custom_size = Some(Vec2::new(
            event.width,
            event.height,
        ));
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Resource)]
pub struct AppConfig {
    window: WindowConfig,
    texture: TextureConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    width: u32,
    height: u32,
    fullscreen: bool,
    resizable: bool,
    vsync: bool,
    override_scale_factor: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            fullscreen: false,
            resizable: false,
            vsync: true,
            override_scale_factor: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextureConfig {
    width: u32,
    height: u32,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            width: 2560,
            height: 1440,
        }
    }
}
