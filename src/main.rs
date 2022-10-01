use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use rand::SeedableRng;

pub mod ui;

/// The random number generator we are using.
pub type Random = rand_xoshiro::Xoroshiro128StarStar;

/// The glorious entry point.
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 948.0,
            height: 533.0,
            ..Default::default()
        })
        .insert_resource(Random::from_entropy())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_scene)
        .add_system(ui::Terminal::animate_system)
        .add_system(keyboard_events)
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

fn keyboard_events(mut key_evr: EventReader<KeyboardInput>, mut windows: ResMut<Windows>) {
    use bevy::input::ButtonState;
    use bevy::window::WindowMode;

    let window = windows.get_primary_mut().unwrap();
    for ev in key_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {}
            ButtonState::Released => {
                if let Some(KeyCode::F) = ev.key_code {
                    window.set_mode(if window.mode() == WindowMode::Windowed {
                        println!("FULLSCREEN");
                        WindowMode::Fullscreen
                    } else {
                        WindowMode::Windowed
                    });
                }
            }
        }
    }
}
