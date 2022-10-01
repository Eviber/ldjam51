use bevy::prelude::*;

use rand::SeedableRng;

pub mod ui;

/// The random number generator we are using.
pub type Random = rand_xoshiro::Xoroshiro128StarStar;

/// The glorious entry point.
fn main() {
    App::new()
        .insert_resource(Random::from_entropy())
        .insert_resource(WindowDescriptor {
            width: 948.0,
            height: 533.0,
            resizable: false,
            ..WindowDescriptor::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_scene)
        .add_system(ui::Terminal::animate_system)
        .add_system(ui::Choice::select_choice_system)
        .add_system_to_stage(CoreStage::PreUpdate, ui::Prev::<Interaction>::update_prev)
        .run();
}

fn setup_scene(mut commands: Commands, assets: Res<AssetServer>) {
    let terminal_font = assets.load("RobotoMono-Medium.ttf");

    commands.spawn_bundle(Camera2dBundle::default());

    let button_text_style = TextStyle {
        color: Color::BLACK,
        font: terminal_font,
        font_size: 24.0,
    };

    commands
        .spawn_bundle(ui::ContainerBundle {
            style: Style {
                flex_grow: 1.0,
                ..Style::default()
            },
            ..ui::ContainerBundle::default()
        })
        .with_children(|children| {
            children
                .spawn_bundle(ui::ContainerBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        position_type: PositionType::Absolute,
                        ..Style::default()
                    },
                    ..ui::ContainerBundle::default()
                })
                .with_children(|children| {
                    children
                        .spawn_bundle(ui::ChoiceBundle {
                            choice: ui::Choice(1),
                            ..ui::ChoiceBundle::default()
                        })
                        .with_children(|children| {
                            children.spawn_bundle(TextBundle {
                                text: Text::from_section(
                                    "trouvez une planette habitée",
                                    button_text_style.clone(),
                                ),
                                ..TextBundle::default()
                            });
                        });
                    children
                        .spawn_bundle(ui::ChoiceBundle {
                            choice: ui::Choice(0),
                            ..ui::ChoiceBundle::default()
                        })
                        .with_children(|children| {
                            children.spawn_bundle(TextBundle {
                                text: Text::from_section(
                                    "spacioport le plus proche. vous avez pas le temps de chercher",
                                    button_text_style.clone(),
                                ),
                                ..TextBundle::default()
                            });
                        });
                });

            children.spawn_bundle(ui::TerminalBundle {
                terminal: ui::Terminal {
                    font: button_text_style.font.clone(),
                    animated_text: String::from("lolilol"),
                    animation_index: 0,
                    animation_period_range: (0.02, 0.04),
                    next_animation_time: 0.0,
                },
                text: TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        align_self: AlignSelf::FlexEnd,
                        ..Style::default()
                    },
                    text: Text {
                        alignment: TextAlignment::BOTTOM_LEFT,
                        ..Text::default()
                    },
                    ..TextBundle::default()
                },
            });
        });
}
