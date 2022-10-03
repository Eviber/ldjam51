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
pub struct AudioFlag(bool);

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
        .insert_resource(AudioFlag(true))
        .insert_resource(Vec::<Handle<AudioSource>>::new())
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_startup_system(setup_scene)
        .add_startup_system(setup_audio)
        .add_system_to_stage(CoreStage::First, ui::Prev::<Interaction>::update_prev)
        .add_system(ui::Terminal::animate_system)
        .add_system(keyboard_events)
        .add_system(Selector::update_system)
        .add_system(ui::Choice::select_choice_system)
        .add_system(audio_game)
        .add_system(story_loop)
        .run();

    ExitCode::SUCCESS
}

fn setup_audio(asset_server: Res<AssetServer>, mut audio_assets: ResMut<Vec::<Handle<AudioSource>>>) {
    audio_assets.push(asset_server.load("mainmenu.ogg")); // 0
    audio_assets.push(asset_server.load("credits.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_drums_club.ogg")); // 2
    audio_assets.push(asset_server.load("GJ_10s_drums_hiphop.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_drums_slowbreak.ogg")); // 4
    audio_assets.push(asset_server.load("GJ_10s_drums_synthwave.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_layers_arp.ogg")); // 6
    audio_assets.push(asset_server.load("GJ_10s_layers_bass1.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_layers_bass2.ogg")); // 8
    audio_assets.push(asset_server.load("GJ_10s_layers_bells.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_layers_chord.ogg")); // 10
    audio_assets.push(asset_server.load("GJ_10s_layers_percussion.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_solo_1.ogg")); // 12
    audio_assets.push(asset_server.load("GJ_10s_solo_2.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_special_club.ogg")); // 14
    audio_assets.push(asset_server.load("GJ_10s_transition_1.ogg"));
    audio_assets.push(asset_server.load("GJ_10s_transition_2_loop.ogg")); // 16
}

fn audio_menu(audio: Res<Audio>, audio_assets: Res<Vec::<Handle<AudioSource>>>) {
    audio.play(audio_assets[0].clone()).loop_from(20.0);
}

fn audio_credits(audio: Res<Audio>, audio_assets: Res<Vec::<Handle<AudioSource>>>) {
    audio.play(audio_assets[1].clone()).looped();
}

fn audio_game(mut audio: ResMut<DynamicAudioChannels>,
        mut audio_flag: ResMut<AudioFlag>,
        executor: Res<story::StoryExecutor>,
        audio_assets: Res<Vec::<Handle<AudioSource>>>) {
    if audio_flag.0 == true {
        audio_flag.0 = false;
        match executor.current_batch {
            0 => {
                match executor.current_prompt {
                    0 => {
                        //audio.create_channel("a1").play(audio_assets[6].clone());
                    },
                    1 => {
                        audio.create_channel("a2").play(audio_assets[5].clone());
                        audio.create_channel("b2").play(audio_assets[6].clone());
                        audio.create_channel("c2").play(audio_assets[7].clone());
                    },
                    2 => {
                        audio.create_channel("a3").play(audio_assets[5].clone());
                        audio.create_channel("b2").play(audio_assets[6].clone());
                        audio.create_channel("c3").play(audio_assets[7].clone());
                    },
                    3 => {
                        audio.create_channel("a4").play(audio_assets[5].clone());
                        audio.create_channel("b4").play(audio_assets[6].clone());
                        audio.create_channel("c4").play(audio_assets[7].clone());
                        audio.create_channel("b2").play(audio_assets[9].clone());
                    },
                    _ => {
                    }
                }
            },
            1 => {
                match executor.current_prompt {
                    0 => {
                        audio.create_channel("a1").play(audio_assets[6].clone());
                    },
                    1 => {
                        audio.create_channel("a2").play(audio_assets[5].clone());
                        //audio.create_channel("b2").play(audio_assets[6].clone());
                        audio.create_channel("c2").play(audio_assets[7].clone());
                    },
                    2 => {
                        audio.create_channel("a3").play(audio_assets[5].clone());
                        //audio.create_channel("b3").play(audio_assets[9].clone());
                        audio.create_channel("c3").play(audio_assets[7].clone());
                    },
                    3 => {
                        audio.create_channel("a4").play(audio_assets[5].clone());
                        //audio.create_channel("b4").play(audio_assets[6].clone());
                        audio.create_channel("c4").play(audio_assets[7].clone());
                    },
                    _ => {
                    }
                }
            },
            _ => {}
        }
    }
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
    mut audio_flag: ResMut<AudioFlag>,
) {
    remaining_time.0 -= dt.delta_seconds();

    if remaining_time.0 > 0.0 {
        return;
    }
    audio_flag.0 = true;
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
