use bevy::prelude::*;
use bevy::ui::FocusPolicy;

/// A **bundle** that represents a button.
#[derive(Bundle, Default, Debug)]
pub struct ChoiceBundle {
    pub node: Node,
    pub button: Button,
    pub style: Style,
    pub interaction: Interaction,
    pub focus_policy: FocusPolicy,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub choice: Choice,
    pub prev_interaction: Prev<Interaction>,
}

/// A **component** that that is added to buttons.
///
/// The index of the selected choice is stored in this component.
#[derive(Component, Default, Debug)]
pub struct Choice(pub usize);

/// Stores the previous value of the component of type `T`.
#[derive(Component, Default, Debug)]
pub struct Prev<T>(pub T);

impl<T: Component + Clone> Prev<T> {
    pub fn update_prev(mut query: Query<(&mut Prev<T>, &T), Changed<T>>) {
        for (mut prev, t) in query.iter_mut() {
            prev.0 = t.clone();
        }
    }
}

impl Choice {
    #[allow(clippy::type_complexity)]
    pub fn select_choice_system(
        mut query: Query<
            (&Interaction, &Prev<Interaction>, &Choice),
            (With<Button>, Changed<Interaction>),
        >,
    ) {
        for (interaction, prev, choice) in query.iter_mut() {
            // Detect a transition from clicked into hovered.
            if matches!(
                (*interaction, prev.0),
                (Interaction::Hovered, Interaction::Clicked)
            ) {
                println!("clicked on {}", choice.0);
            }
        }
    }
}
