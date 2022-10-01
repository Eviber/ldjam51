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
        .add_system(ui::Prompt::input_system)
        .add_system(ui::PromptOptions::update_options)
        .run();
}

fn setup_scene(mut commands: Commands, assets: Res<AssetServer>) {
    let terminal_font = assets.load("RobotoMono-Medium.ttf");

    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(ui::TerminalBundle {
        terminal: ui::Terminal {
            animated_text: String::from("Salut à tous ! Comment ça va ?"),
            animation_index: 0,
            animation_period_range: (0.2, 0.3),
            next_animation_time: 0.5,
            font: terminal_font.clone(),
        },
        ..ui::TerminalBundle::default()
    });
    let prompt_entity = commands
        .spawn_bundle(ui::PromptBundle::new(TextStyle {
            color: Color::WHITE,
            font: terminal_font.clone(),
            font_size: 24.0,
        }))
        .insert(Transform::from_xyz(0.0, 100.0, 0.0))
        .id();

    commands
        .spawn_bundle(ui::PromptOptionsBundle {
            prompt_options: ui::PromptOptions {
                font: terminal_font,
                options: vec![
                    "you should definitely eat those".into(),
                    "nah, better not".into(),
                    "do it you won't".into(),
                ],
                tracked_entity: prompt_entity,
                selected: 0,
            },
            text: Text2dBundle::default(),
        })
        .insert(Transform::from_xyz(0.0, -100.0, 0.0));
}
