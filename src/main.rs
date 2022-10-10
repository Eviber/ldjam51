use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
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
    timer: Entity,
}

/// Game states
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    /// Main menu
    Menu,
    /// Game is running
    Running,
    /// Credits
    Credits,
}

/// Menu entity
struct Menu(Entity);

/// Credits entity
struct Credits(Entity);

/// Current background music
pub struct BackgroundMusic(Handle<AudioInstance>);

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

    let running_state = SystemSet::on_update(GameState::Running)
        .with_system(ui::Prev::<Interaction>::update_prev)
        .with_system(ui::Terminal::animate_system)
        .with_system(Selector::update_system)
        .with_system(ui::Choice::select_choice_system)
        .with_system(audio_game)
        .with_system(story_loop)
        .with_system(update_timer);

    App::new()
        .insert_resource(WindowDescriptor {
            title: "PROXIMA".to_string(),
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
        .add_state(GameState::Menu)
        .add_startup_system(setup_audio)
        .add_startup_system(load_assets)
        .add_system_set(
            SystemSet::on_enter(GameState::Menu)
                .with_system(menu_setup)
                .with_system(audio_menu),
        )
        .add_system_set(SystemSet::on_update(GameState::Menu).with_system(menu_update))
        .add_system_set(
            SystemSet::on_enter(GameState::Credits)
                .with_system(credits_setup)
                .with_system(audio_credits),
        )
        .add_system_set(SystemSet::on_exit(GameState::Credits).with_system(audio_stop))
        .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(audio_stop))
        .add_system_set(SystemSet::on_update(GameState::Credits).with_system(credits_update))
        .add_system_set(SystemSet::on_enter(GameState::Running).with_system(setup_scene))
        .add_system_to_stage(CoreStage::First, ui::Prev::<Interaction>::update_prev)
        .add_system_set(running_state)
        .add_system(keyboard_events)
        .run();

    ExitCode::SUCCESS
}

fn setup_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut audio_assets: ResMut<Vec<Handle<AudioSource>>>,
) {
    commands.spawn_bundle(Camera2dBundle::default()); // whack

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

    commands.insert_resource(BackgroundMusic(
        audio.play(audio_assets[0].clone()).loop_from(20.0).handle(),
    ));
    audio.stop();
}

// pub struct BackgroundMusic(Handle<AudioInstance>);

/// Starts the audio for the menu
fn audio_menu(
    audio: Res<Audio>,
    audio_assets: Res<Vec<Handle<AudioSource>>>,
    mut bgm: ResMut<BackgroundMusic>,
) {
    if let Some(handle) = audio_assets.get(0) {
        bgm.0 = audio.play(handle.clone()).loop_from(20.0).handle();
    }
}

/// Starts the audio for the credits
fn audio_credits(
    audio: Res<Audio>,
    audio_assets: Res<Vec<Handle<AudioSource>>>,
    mut bgm: ResMut<BackgroundMusic>,
) {
    if let Some(handle) = audio_assets.get(1) {
        bgm.0 = audio.play(handle.clone()).looped().handle();
    }
}

/// stops the audio when changing state
fn audio_stop(audio: Res<Audio>) {
    audio.stop();
}

const VOLUME: f64 = 0.05;

fn audio_game(
    mut audio: ResMut<DynamicAudioChannels>,
    mut audio_flag: ResMut<AudioFlag>,
    executor: Res<story::StoryExecutor>,
    audio_assets: Res<Vec<Handle<AudioSource>>>,
) {
    if audio_flag.0 == true {
        audio_flag.0 = false;
        match executor.current_batch {
            0 => {
                match executor.current_prompt {
                    0 => {
                        //audio.create_channel("a1").play(audio_assets[6].clone());
                    }
                    1 => {
                        audio
                            .create_channel("a2")
                            .play(audio_assets[7].clone())
                            .with_volume(VOLUME);
                    }
                    2 => {
                        audio
                            .create_channel("a3")
                            .play(audio_assets[3].clone())
                            .with_volume(VOLUME);
                        audio
                            .create_channel("c3")
                            .play(audio_assets[7].clone())
                            .with_volume(VOLUME);
                    }
                    3 => {
                        audio
                            .create_channel("a4")
                            .play(audio_assets[3].clone())
                            .with_volume(VOLUME);
                        audio
                            .create_channel("b4")
                            .play(audio_assets[7].clone())
                            .with_volume(VOLUME);
                    }
                    _ => {}
                }
            }
            1 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b2")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b3")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            2 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b2")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d2")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            3 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b2")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                4 => {
                    audio
                        .create_channel("a5")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b5")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c5")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            4 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[12].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                4 => {
                    audio
                        .create_channel("a5")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b5")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c5")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d5")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                5 => {
                    audio
                        .create_channel("a6")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b6")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c6")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            5 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b2")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                4 => {
                    audio
                        .create_channel("a5")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b5")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c5")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            6 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b2")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[5].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            7 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b1")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c1")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b2")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b4")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                4 => {
                    audio
                        .create_channel("a5")
                        .play(audio_assets[4].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b5")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c5")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            8 => match executor.current_prompt {
                0 => {
                    audio
                        .create_channel("a1")
                        .play(audio_assets[13].clone())
                        .with_volume(VOLUME);
                }
                1 => {
                    audio
                        .create_channel("a2")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c2")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                2 => {
                    audio
                        .create_channel("a3")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c3")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                }
                3 => {
                    audio
                        .create_channel("a4")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c4")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d4")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                4 => {
                    audio
                        .create_channel("a5")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c5")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d5")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                5 => {
                    audio
                        .create_channel("a6")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c6")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d6")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                6 => {
                    audio
                        .create_channel("a7")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b7")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c7")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d7")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                7 => {
                    audio
                        .create_channel("a8")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b8")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c8")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d8")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                8 => {
                    audio
                        .create_channel("a9")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b9")
                        .play(audio_assets[10].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c9")
                        .play(audio_assets[8].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d9")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                9 => {
                    audio
                        .create_channel("a10")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b10")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c10")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d10")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                10 => {
                    audio
                        .create_channel("a11")
                        .play(audio_assets[3].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("b11")
                        .play(audio_assets[6].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("c11")
                        .play(audio_assets[7].clone())
                        .with_volume(VOLUME);
                    audio
                        .create_channel("d11")
                        .play(audio_assets[9].clone())
                        .with_volume(VOLUME);
                }
                _ => {}
            },
            // 8 => match executor.current_prompt {
            //     0 => {
            //         audio
            //             .create_channel("a1")
            //             .play(audio_assets[12].clone())
            //             .with_volume(VOLUME);
            //     }
            //     1 => {
            //         audio
            //             .create_channel("a2")
            //             .play(audio_assets[5].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("b2")
            //             .play(audio_assets[8].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("c2")
            //             .play(audio_assets[10].clone())
            //             .with_volume(VOLUME);
            //     }
            //     2 => {
            //         audio
            //             .create_channel("a3")
            //             .play(audio_assets[5].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("b3")
            //             .play(audio_assets[8].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("c3")
            //             .play(audio_assets[10].clone())
            //             .with_volume(VOLUME);
            //     }
            //     3 => {
            //         audio
            //             .create_channel("a4")
            //             .play(audio_assets[5].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("c4")
            //             .play(audio_assets[7].clone())
            //             .with_volume(VOLUME);
            //     }
            //     4 => {
            //         audio
            //             .create_channel("a5")
            //             .play(audio_assets[5].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("c5")
            //             .play(audio_assets[7].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("d5")
            //             .play(audio_assets[9].clone())
            //             .with_volume(VOLUME);
            //     }
            //     5 => {
            //         audio
            //             .create_channel("a6")
            //             .play(audio_assets[5].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("c6")
            //             .play(audio_assets[7].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("d6")
            //             .play(audio_assets[9].clone())
            //             .with_volume(VOLUME);
            //     }
            //     6 => {
            //         audio
            //             .create_channel("a7")
            //             .play(audio_assets[5].clone())
            //             .with_volume(VOLUME);
            //         audio
            //             .create_channel("d7")
            //             .play(audio_assets[6].clone())
            //             .with_volume(VOLUME);
            //     }
            //     _ => {}
            // },
            _ => {}
        }
    }
}

struct ImagesHandle {
    bg: Handle<Image>,
    menu: Handle<Image>,
    credits: Handle<Image>,
    glass: Handle<Image>,
    neon: Handle<Image>,
    select_marker: Handle<Image>,
    bar: Handle<Image>,
    timer: Handle<Image>,
}

struct FontHandle(Handle<Font>);

fn load_assets(mut commands: Commands, assets: Res<AssetServer>) {
    let images_assets = ImagesHandle {
        bg: assets.load("BackgroundStarsLoop.png"),
        menu: assets.load("MenuText.png"),
        credits: assets.load("Credits.png"),
        glass: assets.load("glass.png"),
        neon: assets.load("NewNeonFrame.png"),
        select_marker: assets.load("select_marker.png"),
        bar: assets.load("bar.png"),
        timer: assets.load("Timer.png"),
    };

    commands.insert_resource(images_assets);
    commands.insert_resource(FontHandle(assets.load("RobotoMono-Medium.ttf")));
}

const BAR_W: f32 = 191.0;
const BAR_X: f32 = 26.0;
const BAR_Y: f32 = 11.0;
const BAR_H: f32 = 16.0;
const CHOICE_X1: f32 = 230.0;
const CHOICE_Y1: f32 = 335.0;
const CHOICE_X2: f32 = 230.0;
const CHOICE_Y2: f32 = 407.0;

struct BackgroundEntity(Entity);

fn setup_scene(
    mut commands: Commands,
    story: Res<story::StoryExecutor>,
    images: Res<ImagesHandle>,
    terminal_font: Res<FontHandle>,
) {
    let terminal_font = terminal_font.0.clone();

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

    let style = Style {
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        position_type: PositionType::Absolute,
        ..default()
    };

    let mut choice1 = Entity::from_raw(0); // TODO remove this hack
    let mut choice2 = Entity::from_raw(0); // TODO remove this hack // TODO: don't remove it it's cool // TODO ok maybe don't remove it
    let mut terminal = Entity::from_raw(0); // TODO remove this hack
    let mut timer = Entity::from_raw(0); // TODO remove this hack

    commands
        .spawn_bundle(ImageBundle {
            style: style.clone(),
            image: UiImage(images.bg.clone()),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: style.clone(),
                    image: UiImage(images.glass.clone()),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(ImageBundle {
                        style,
                        image: UiImage(images.neon.clone()),
                        ..default()
                    });
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    top: Val::Percent(0.0),
                                    left: Val::Percent(460.0 / 1896.0 * 100.0),
                                    ..default()
                                },
                                size: Size::new(Val::Px(490.0), Val::Px(60.0)),
                                ..default()
                            },
                            image: UiImage(images.select_marker.clone()),
                            color: UiColor(Color::rgba(1.0, 1.0, 1.0, 0.2)),
                            ..default()
                        })
                        .insert(Selector);
                    timer = parent
                        .spawn_bundle(ImageBundle {
                            style: Style {
                                size: Size::new(Val::Px(BAR_W), Val::Px(BAR_H)),
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    left: Val::Px(488.0 + BAR_X),
                                    top: Val::Px(275.0 + BAR_Y),
                                    ..default()
                                },
                                ..default()
                            },
                            image: UiImage(images.bar.clone()),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(ImageBundle {
                                style: Style {
                                    size: Size::new(Val::Px(484.0 / 2.0), Val::Px(76.0 / 2.0)),
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        left: Val::Px(0.0 - BAR_X),
                                        top: Val::Px(0.0 - BAR_Y),
                                        ..default()
                                    },
                                    ..default()
                                },
                                image: UiImage(images.timer.clone()),
                                ..default()
                            });
                        })
                        .id();
                });

            parent
                .spawn_bundle(ui::ChoiceBundle {
                    choice: ui::Choice(1),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(CHOICE_X1),
                            top: Val::Px(CHOICE_Y1),
                            ..default()
                        },
                        size: Size::new(Val::Px(490.0), Val::Px(60.0)),
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
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        ..default()
                                    },
                                    max_size: Size::new(Val::Px(490.0), Val::Px(60.0)),
                                    ..default()
                                },
                                ..default()
                            },
                        })
                        .id();
                });

            parent
                .spawn_bundle(ui::ChoiceBundle {
                    choice: ui::Choice(2),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(CHOICE_X2),
                            top: Val::Px(CHOICE_Y2),
                            ..default()
                        },
                        size: Size::new(Val::Px(490.0), Val::Px(60.0)),
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
                            text: TextBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        ..default()
                                    },
                                    max_size: Size::new(Val::Px(490.0), Val::Px(60.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        })
                        .id();
                });

            terminal = parent
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
        });

    commands.insert_resource(UiElements {
        terminal,
        choices: [choice1, choice2],
        timer,
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
    mut state: ResMut<State<GameState>>,
) {
    remaining_time.0 -= dt.delta_seconds();

    if remaining_time.0 > 0.0 {
        return;
    }
    audio_flag.0 = true;
    remaining_time.0 = 10.0;
    let next_prompt = match executor.select_answer(current_selection.0, &mut *random) {
        Some(prompt) => prompt,
        None => {
            state.set(GameState::Menu).unwrap();
            return;
        }
    };

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
                #[cfg(debug_assertions)]
                Some(KeyCode::Space) => {
                    time.0 += 5.0;
                }
                #[cfg(debug_assertions)]
                Some(KeyCode::Return) => {
                    time.0 = 0.0;
                }
                _ => {}
            },
        }
    }
}

fn update_timer(
    timer: ResMut<RemainingTime>,
    ui_elements: Res<UiElements>,
    mut ui_query: Query<&mut Style>,
) {
    let mut bar = ui_query.get_mut(ui_elements.timer).unwrap();
    bar.size.width = Val::Px(BAR_W * timer.0 / 10.0);
}

fn menu_setup(mut commands: Commands, images: Res<ImagesHandle>) {
    let style = Style {
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        position_type: PositionType::Absolute,
        ..default()
    };

    let mut menu = Entity::from_raw(0);
    let bg = commands
        .spawn_bundle(ImageBundle {
            style: style.clone(),
            image: UiImage(images.bg.clone()),
            ..default()
        })
        .with_children(|parent| {
            menu = parent.spawn_bundle(ImageBundle {
                style: style.clone(),
                image: UiImage(images.menu.clone()),
                ..default()
            }).id()
            // TODO: add button here
            ;
        })
        .id();
    commands.insert_resource(Menu(menu));
    commands.insert_resource(BackgroundEntity(bg));
}

/// Menu update system
/// checks for mouse clicks and changes the scene if the mouse is clicked
/// after hiding the menu
fn menu_update(
    mut state: ResMut<State<GameState>>,
    mut buttons: ResMut<Input<MouseButton>>,
    menu: Res<Menu>,
    mut query: Query<&mut Style>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        state.set(GameState::Running).unwrap();
        buttons.reset(MouseButton::Left);
        println!("Credits");
        query.get_mut(menu.0).unwrap().display = Display::None;
    }
}

/// Credits setup system
fn credits_setup(mut commands: Commands, images: Res<ImagesHandle>) {
    let style = Style {
        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
        position_type: PositionType::Absolute,
        ..default()
    };

    let mut credits = Entity::from_raw(0);
    commands
        .spawn_bundle(ImageBundle {
            style: style.clone(),
            image: UiImage(images.bg.clone()),
            ..default()
        })
        .with_children(|parent| {
            credits = parent
                .spawn_bundle(ImageBundle {
                    style: style.clone(),
                    image: UiImage(images.credits.clone()),
                    ..default()
                })
                .id();
        });
    commands.insert_resource(Credits(credits));
}

fn credits_update(
    mut state: ResMut<State<GameState>>,
    mut buttons: ResMut<Input<MouseButton>>,
    credits: ResMut<Credits>,
    mut query: Query<&mut Style>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        state.set(GameState::Menu).unwrap();
        buttons.reset(MouseButton::Left);
        println!("Menu");
        query.get_mut(credits.0).unwrap().display = Display::None;
    }
}
