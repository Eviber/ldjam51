use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use bevy::ui::FocusPolicy;
use rand::SeedableRng;

use std::process::ExitCode;

mod parsing;
mod story;
mod ui;

/// The random number generator we are using.
pub type Random = rand_xoshiro::Xoroshiro128StarStar;

/// Index of the selected answer
pub struct CurrentSelection(usize);

/// Remaining time to answer (in seconds)
pub struct RemainingTime(f32);

/// The glorious entry point.
fn main() -> ExitCode {
    let p = match parsing::read_config() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading config: {e}");
            return ExitCode::FAILURE;
        }
    };

    let story = match story::parse_story("assets/story.json".as_ref()) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    App::new()
        .insert_resource(WindowDescriptor {
            title: "PLACEHOLDER".to_string(),
            width: p.window_size.width,
            height: p.window_size.height,
            resizable: false,
            ..default()
        })
        .insert_resource(CurrentSelection(0))
        .insert_resource(RemainingTime(10.0))
        .insert_resource(story::StoryExecutor::from(story))
        .insert_resource(Random::from_entropy())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_scene)
        .add_system_to_stage(CoreStage::First, ui::Prev::<Interaction>::update_prev)
        .add_system(ui::Terminal::animate_system)
        .add_system(keyboard_events)
        .add_system(ui::Choice::select_choice_system)
        .run();

    ExitCode::SUCCESS
}

fn setup_scene(mut commands: Commands, assets: Res<AssetServer>) {
    let terminal_font = assets.load("RobotoMono-Medium.ttf");

    commands.spawn_bundle(Camera2dBundle::default());

    let query_text_style = TextStyle {
        color: Color::WHITE,
        font: terminal_font.clone(),
        font_size: 34.0,
    };

    let button_text_style = TextStyle {
        color: Color::WHITE,
        font: terminal_font,
        font_size: 24.0,
    };

    commands.spawn_bundle(ImageBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Absolute,
            ..default()
        },
        image: UiImage(assets.load("bg.jpg")),
        ..default()
    });

    commands.spawn_bundle(ui::TerminalBundle {
        terminal: ui::Terminal {
            style: query_text_style.clone(),
            animated_text: String::from(
                "An asteroid is going to hit us!\nQuick, what should we do?",
            ),
            animation_index: 0,
            animation_period_range: (0.02, 0.04),
            next_animation_time: 0.0,
        },
        text: TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(460.0 / 1896.0 * 100.0),
                    top: Val::Percent(200.0 / 1066.0 * 100.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                alignment: TextAlignment::BOTTOM_LEFT,
                ..default()
            },
            ..default()
        },
    });

    // The first `ChoiceBundle` seems to be ignored by the input system. We don't have the time
    // to understand why. Quick fix: add an empty choice button at the begining.
    commands.spawn_bundle(ui::ChoiceBundle::default());

    commands
        .spawn_bundle(ui::ChoiceBundle {
            choice: ui::Choice(2),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(460.0 / 1896.0 * 100.0),
                    top: Val::Percent(860.0 / 1066.0 * 100.0),
                    ..default()
                },
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            ..default()
        })
        .with_children(|children| {
            children.spawn_bundle(ui::TerminalBundle {
                terminal: ui::Terminal {
                    style: button_text_style.clone(),
                    animated_text: String::from("Change direction to avoid it"),
                    animation_index: 0,
                    animation_period_range: (0.02, 0.04),
                    next_animation_time: 0.0,
                },
                ..default()
            });
        });

    commands
        .spawn_bundle(ui::ChoiceBundle {
            choice: ui::Choice(1),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(460.0 / 1896.0 * 100.0),
                    top: Val::Percent(770.0 / 1066.0 * 100.0),
                    ..default()
                },
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            ..default()
        })
        .with_children(|children| {
            children.spawn_bundle(ui::TerminalBundle {
                terminal: ui::Terminal {
                    style: button_text_style.clone(),
                    animated_text: String::from("Send a rocket and explode it"),
                    animation_index: 0,
                    animation_period_range: (0.02, 0.04),
                    next_animation_time: 0.0,
                },
                ..default()
            });
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
                        WindowMode::Fullscreen
                    } else {
                        WindowMode::Windowed
                    });
                }
            }
        }
    }
}
