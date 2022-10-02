use bevy::prelude::*;

/// A **resource** that represent the user's current selection.
pub struct CurrentSelection(pub usize);

/// A marker component for the entity that's responsible for selecting a button.
#[derive(Clone, Copy, Component, Debug)]
pub struct Selector;

impl Selector {
    pub fn update_system(
        mut query: Query<&mut Style, With<Selector>>,
        current: Res<CurrentSelection>,
    ) {
        if !current.is_changed() {
            return;
        }

        for mut style in query.iter_mut() {
            match current.0 {
                0 => style.position.top = Val::Percent(-1000.0),
                1 => style.position.top = Val::Percent(770.0 / 1066.0 * 100.0),
                2 => style.position.top = Val::Percent(860.0 / 1066.0 * 100.0),
                _ => (),
            }
        }
    }
}
