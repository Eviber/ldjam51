use bevy::prelude::*;

use rand::SeedableRng;

pub mod ui;

/// The random number generator we are using.
pub type Random = rand_xoshiro::Xoroshiro128StarStar;

/// The glorious entry point.
fn main() {
    App::new()
        .insert_resource(Random::from_entropy())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_scene)
        .add_system(ui::Terminal::animate_system)
        .run();
}

fn setup_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(ui::TerminalBundle {
        terminal: ui::Terminal {
            animated_text: String::from("Hello, World!"),
            animation_index: 0,
            animation_period_range: (0.1, 0.3),
            next_animation_time: 0.5,
            font: assets.load("RobotoMono-Medium.ttf"),
        },
        ..ui::TerminalBundle::default()
    });
}
