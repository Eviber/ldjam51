use bevy::prelude::*;
use fuzzy_matcher::skim::{SkimMatcherV2, SkimScoreConfig};
use fuzzy_matcher::FuzzyMatcher;

#[derive(Default, Bundle)]
pub struct PromptBundle {
    #[bundle]
    pub text: Text2dBundle,
    pub prompt: Prompt,
}

impl PromptBundle {
    pub fn new(style: TextStyle) -> Self {
        Self {
            text: Text2dBundle {
                text: Text::from_section(String::new(), style),
                ..Text2dBundle::default()
            },
            prompt: Prompt::default(),
        }
    }
}

#[derive(Bundle)]
pub struct PromptOptionsBundle {
    #[bundle]
    pub text: Text2dBundle,
    pub prompt_options: PromptOptions,
}

/// Displays the options available in a prompt.
#[derive(Component)]
pub struct PromptOptions {
    /// The entity that is being tracked.
    ///
    /// That entity is supposed to have a [`Text`] component.
    pub tracked_entity: Entity,
    /// The options available.
    pub options: Vec<String>,
    /// The font used in the options.
    pub font: Handle<Font>,
    /// The option that is currently selected.
    pub selected: usize,
}

/// The local state of the `PromptOptions::update_options` system.
pub struct UpdateOptionsLocal {
    matcher: SkimMatcherV2,
    buffer: Vec<(usize, i64)>,
}

impl Default for UpdateOptionsLocal {
    fn default() -> Self {
        Self {
            matcher: SkimMatcherV2::default().use_cache(true).ignore_case(),
            buffer: Vec::new(),
        }
    }
}

impl PromptOptions {
    /// A **system** that's responsible for updating the [`Text`] component associated with a
    /// [`PromptOptions`] component.
    pub fn update_options(
        mut this: Local<UpdateOptionsLocal>,
        mut query: Query<(&mut PromptOptions, &mut Text), Without<Prompt>>,
        tracked_query: Query<&Text, (Changed<Text>, With<Prompt>)>,
    ) {
        let this = &mut *this;

        for (mut prompt_options, mut text) in query.iter_mut() {
            if let Ok(tracked_text) = tracked_query.get(prompt_options.tracked_entity) {
                let prompt = tracked_text.sections.last().unwrap().value.trim();

                // Populate the buffer with the possible matches of the user's input.
                this.buffer.clear();
                this.buffer
                    .extend(
                        prompt_options
                            .options
                            .iter()
                            .enumerate()
                            .filter_map(|(i, opt)| {
                                this.matcher
                                    .fuzzy_match(opt, prompt)
                                    .map(|score| (i, score))
                            }),
                    );
                // Sort the buffer to get the best matches at the end of the buffer.
                this.buffer.sort_unstable_by_key(|&(_, score)| score);

                if let Some(&(i, _)) = this.buffer.first() {
                    prompt_options.selected = i;
                }

                // Update existing sections (to having unnecessary allocations).
                while text.sections.len() < this.buffer.len() {
                    text.sections.push(TextSection {
                        value: String::new(),
                        style: TextStyle {
                            color: Color::WHITE,
                            font: prompt_options.font.clone(),
                            font_size: 24.0,
                        },
                    });
                }

                // Write the text with the new sections.
                let mut sections = text.sections.iter_mut();
                for &(index, _) in &this.buffer {
                    let section = sections.next().unwrap();
                    section.value.clear();
                    section.value.push_str(&prompt_options.options[index]);
                    section.value.push_str("\n");

                    if index == prompt_options.selected {
                        section.style.color = Color::BLACK;
                    } else {
                        section.style.color = Color::WHITE;
                    }
                }
                for section in sections {
                    section.value.clear();
                }
            }
        }
    }
}

/// A **component** that uses the user's inputs to populate a text-box.
#[derive(Default, Component)]
pub struct Prompt {
    /// The cursor within the string.
    pub cursor: usize,
}

#[derive(Default)]
pub struct InputSystemLocal {
    /// The gather character on the current frame.
    buffer: String,
}

impl Prompt {
    /// This **system** gathers the user's inputs and applies them to the prompt.
    pub fn input_system(
        mut this: Local<InputSystemLocal>,
        mut characters: EventReader<ReceivedCharacter>,
        mut query: Query<(&mut Prompt, &mut Text)>,
    ) {
        this.buffer.clear();

        let mut backspace_count: u32 = 0;
        let mut confirm_pressed: bool = false;

        // Collect the characters that were pressed so far.
        for c in characters.iter() {
            match c.char {
                '\u{08}' => backspace_count += 1,
                '\n' | '\r' => confirm_pressed = true,
                c if c.is_control() => (),
                c => this.buffer.push(c),
            }
        }

        for (mut prompt, mut text) in query.iter_mut() {
            prompt.cursor += this.buffer.len();
            let section = text.sections.last_mut().unwrap();

            // Append new characters to the prompt.
            section.value.push_str(&this.buffer);

            // Remove the correct number of characters (that include multi-byte) charactes like
            // 'é'.
            for _ in 0..backspace_count {
                if let Some(c) = section.value.pop() {
                    prompt.cursor -= c.len_utf8();
                }
            }

            // If the ENTER key has been pressed, clear the prompt and possibly do something.
            if confirm_pressed {
                section.value.clear();
            }
        }
    }
}
