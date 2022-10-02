use bevy::prelude::*;

use rand::Rng;

use crate::Random;

/// A **bundle** that contains the necessary components to spawn a working terminal.
#[derive(Bundle, Default)]
pub struct TerminalBundle {
    #[bundle]
    pub text: TextBundle,
    pub terminal: Terminal,
}

/// A **component** that allows the user to use a terminal-like interface.
#[derive(Component, Default)]
pub struct Terminal {
    /// The text currently being animated (appended one character after the other).
    pub animated_text: String,
    /// The index of the next character that will be displayed.
    pub animation_index: usize,
    /// The amount of time before another character is displayed.
    pub next_animation_time: f32,
    /// The minimum and maximum amount of time between animated characters. Each time a character
    /// is appended to the terminal, the `next_animation_time` field will be set to a random value
    /// in that range.
    pub animation_period_range: (f32, f32),
    /// The font of the terminal.
    pub style: TextStyle,
}

impl Terminal {
    /// Returns whether this [`Terminal`] is done being animated.
    #[inline]
    pub fn is_done_animating(&self) -> bool {
        self.animation_index == self.animated_text.len()
    }

    /// Returns the next character that should be appended to the terminal.
    #[inline]
    pub fn next_character(&mut self) -> Option<char> {
        let c = self.animated_text[self.animation_index..].chars().next()?;
        self.animation_index += c.len_utf8();
        Some(c)
    }

    /// A **system** that animates [`Terminal`] components.
    pub fn animate_system(
        mut query: Query<(&mut Terminal, &mut Text)>,
        time: Res<Time>,
        mut rng: ResMut<Random>,
    ) {
        let dt = time.delta_seconds();

        for (mut terminal, mut text) in query.iter_mut() {
            // Skip `Terminal` components that are not being animated.
            if terminal.is_done_animating() {
                continue;
            }

            terminal.next_animation_time -= dt;
            while terminal.next_animation_time <= 0.0 {
                // Another character has to be appended this frame.
                let (min, max) = terminal.animation_period_range;
                terminal.next_animation_time += rng.gen_range(min..max);

                // Append a character.
                let c = match terminal.next_character() {
                    Some(c) => c,
                    None => break,
                };

                let last_section = match text.sections.last_mut() {
                    Some(some) => some,
                    None => {
                        text.sections
                            .push(TextSection::from_style(terminal.style.clone()));
                        text.sections.last_mut().unwrap()
                    }
                };

                last_section.value.push(c);
            }
        }
    }
}
