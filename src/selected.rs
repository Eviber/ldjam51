use bevy::prelude::*;

/// A **resource** that represent the user's current selection.
pub struct CurrentSelection(pub usize);

/// A marker component for the entity that's responsible for selecting a button.
#[derive(Clone, Copy, Component, Debug)]
pub struct Selector;

const CHOICE_Y1: f32 = 335.0; // TODO: make good code for once
const CHOICE_Y2: f32 = 407.0;

impl Selector {
    pub fn update_system(
        mut query: Query<&mut Style, With<Selector>>,
        current: Res<CurrentSelection>,
    ) {
        if !current.is_changed() {
            return;
        }

        for mut style in query.iter_mut() {
            println!("selected: {}", current.0);
            match current.0 {
                0 => style.position.top = Val::Percent(-1000.0),
                1 => style.position.top = Val::Px(CHOICE_Y1 + 5.0),
                2 => style.position.top = Val::Px(CHOICE_Y2 + 5.0),
                _ => (),
            }
        }
    }
}
