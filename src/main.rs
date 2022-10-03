use bevy::prelude::*;
use bevy::{input::keyboard::KeyboardInput, ui::FocusPolicy};
use bevy_kira_audio::prelude::*;

use rand::SeedableRng;

use std::process::ExitCode;

mod parsing;
mod selected;
mod story;
mod ui;

use selected::{CurrentSelection, Selector};

/// The random number generator we are using.
pub type Random = rand_xoshiro::Xoroshiro128StarStar;

/// Remaining time to answer (in seconds)
pub struct RemainingTime(f32);

/// Resource referencing every ui element
struct UiElements {
    terminal: Entity,
    choices: [Entity; 2],
}

/// The glorious entry point.
fn main() -> ExitCode {
    let p = match parsing::read_config() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error reading config: {e}");
            return ExitCode::FAILURE;
        }
    };

    let story = match story::parse_story() {
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
        .add_plugin(AudioPlugin)
        .add_startup_system(setup_scene)
        .add_startup_system(play_audio)
        .add_system_to_stage(CoreStage::First, ui::Prev::<Interaction>::update_prev)
        .add_system(ui::Terminal::animate_system)
        .add_system(keyboard_events)
        .add_system(Selector::update_system)
        .add_system(ui::Choice::select_choice_system)
        .add_system(story_loop)
        .run();

    ExitCode::SUCCESS
}

fn play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("mainmenu.ogg"))
        // The first 0.5 seconds will not be looped and are the "intro"
        .loop_from(20.0);
    // Fade-in with a dynamic easing
    // .fade_in(AudioTween::new( Duration::from_secs(2), AudioEasing::OutPowi(2),))
}

fn setup_scene(mut commands: Commands, assets: Res<AssetServer>, story: Res<story::StoryExecutor>) {
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

    let prompt = story.get_current_prompt().unwrap();

    commands.spawn_bundle(ImageBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            position_type: PositionType::Absolute,
            ..default()
        },
        image: UiImage(assets.load("bg.jpg")),
        ..default()
    });

    let terminal = commands
        .spawn_bundle(ui::TerminalBundle {
            terminal: ui::Terminal {
                style: query_text_style,
                animated_text: prompt.request.clone(),
                animation_index: 0,
                animation_period_range: (0.02, 0.04),
                next_animation_time: 0.0,
            },
            text: TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Percent(460.0 / 1896.0 * 100.0),
                        top: Val::Px(85.0),
                        ..default()
                    },
                    max_size: Size::new(Val::Px(460.0), Val::Px(250.0)),
                    ..default()
                },
                ..default()
            },
        })
        .id();

    commands.spawn_bundle(ui::ContainerBundle::default());

    commands
        .spawn_bundle(ImageBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(0.0),
                    left: Val::Percent(460.0 / 1896.0 * 100.0),
                    ..default()
                },
                size: Size::new(Val::Px(490.0), Val::Px(28.0)),
                ..default()
            },
            image: UiImage(assets.load("select_marker.png")),
            color: UiColor(Color::rgba(1.0, 1.0, 1.0, 0.2)),
            focus_policy: FocusPolicy::Pass,
            ..default()
        })
        .insert(Selector);

    let mut choice1 = Entity::from_raw(0); // TODO remove this hack
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
                size: Size::new(Val::Px(490.0), Val::Px(28.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|children| {
            choice1 = children
                .spawn_bundle(ui::TerminalBundle {
                    terminal: ui::Terminal {
                        style: button_text_style.clone(),
                        animated_text: prompt.answers[1].text.clone(),
                        animation_index: 0,
                        animation_period_range: (0.02, 0.04),
                        next_animation_time: 0.0,
                    },
                    text: TextBundle {
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    },
                })
                .id();
        });

    let mut choice2 = Entity::from_raw(0); // TODO remove this hack // TODO: don't remove it it's cool // TODO ok maybe don't remove it
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
                size: Size::new(Val::Px(490.0), Val::Px(28.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|children| {
            choice2 = children
                .spawn_bundle(ui::TerminalBundle {
                    terminal: ui::Terminal {
                        style: button_text_style.clone(),
                        animated_text: prompt.answers[2].text.clone(),
                        animation_index: 0,
                        animation_period_range: (0.02, 0.04),
                        next_animation_time: 0.0,
                    },
                    ..default()
                })
                .id();
        });

    commands.insert_resource(UiElements {
        terminal,
        choices: [choice1, choice2],
    });
}

fn story_loop(
    mut executor: ResMut<story::StoryExecutor>,
    mut current_selection: ResMut<CurrentSelection>,
    mut remaining_time: ResMut<RemainingTime>,
    mut random: ResMut<Random>,
    ui_elements: ResMut<UiElements>,
    dt: Res<Time>,
    mut query: Query<(&mut ui::Terminal, &mut Text)>,
) {
    remaining_time.0 -= dt.delta_seconds();

    if remaining_time.0 > 0.0 {
        return;
    }
    remaining_time.0 = 10.0;
    let next_prompt = executor
        .select_answer(current_selection.0, &mut *random)
        .unwrap();
    current_selection.0 = 0;
    let (mut terminal, mut text) = query.get_mut(ui_elements.terminal).unwrap();
    terminal.animated_text = next_prompt.request.clone();
    terminal.animation_index = 0;
    text.sections
        .iter_mut()
        .map(|s| &mut s.value)
        .for_each(String::clear);
    for (i, choice) in ui_elements.choices.iter().enumerate() {
        let (mut choice, mut text) = query.get_mut(*choice).unwrap();
        choice.animated_text = next_prompt.answers[i + 1].text.clone();
        choice.animation_index = 0;
        text.sections
            .iter_mut()
            .map(|s| &mut s.value)
            .for_each(String::clear);
    }
}

fn keyboard_events(
    mut key_evr: EventReader<KeyboardInput>,
    mut windows: ResMut<Windows>,
    mut time: ResMut<RemainingTime>,
) {
    use bevy::input::ButtonState;
    use bevy::window::WindowMode;

    let window = windows.get_primary_mut().unwrap();
    for ev in key_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {}
            ButtonState::Released => match ev.key_code {
                Some(KeyCode::F) => {
                    window.set_mode(if window.mode() == WindowMode::Windowed {
                        WindowMode::Fullscreen
                    } else {
                        WindowMode::Windowed
                    });
                }
                Some(KeyCode::Space) => {
                    time.0 += 5.0;
                }
                Some(KeyCode::Return) => {
                    time.0 = 0.0;
                }
                _ => {}
            },
        }
    }
}
