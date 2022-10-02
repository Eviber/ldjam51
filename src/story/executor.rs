use rand::{Rng, RngCore};

use super::{Prompt, Story, Variables};

/// A **resource** that's responsible for executing the story's logic.
pub struct StoryExecutor {
    story: Story,
    current_batch: usize,
    current_prompt: usize,
    variables: Variables,
}

impl StoryExecutor {
    /// Gets the current prompt.
    pub fn get_current_prompt(&self) -> Option<&Prompt> {
        let batch = self.story.batches.get(self.current_batch)?;
        Some(&batch.prompts[self.current_prompt])
    }

    /// Selects a specific answer.
    pub fn select_answer(&mut self, choice: usize, rng: &mut dyn RngCore) -> Option<&Prompt> {
        for action in self.story.batches[self.current_batch].prompts[self.current_prompt].answers
            [choice]
            .actions
            .iter()
        {
            let val = self.variables.get_mut(&action.name);
            action.op.execute(val, action.value);
        }

        // Find out which prompt should be used next.
        loop {
            let batch = self.story.batches.get_mut(self.current_batch)?;

            if self.current_prompt >= batch.prompts.len() {
                // The batch has been exhausted.
                self.current_batch += 1;
                continue;
            }

            if batch.randomized {
                // Swap any element with the current first one.
                let swapped_index = rng.gen_range(self.current_prompt..batch.prompts.len());
                batch.prompts.swap(self.current_prompt, swapped_index);
            }

            let prompt = &batch.prompts[self.current_prompt];

            if let Some(pre_condition) = prompt.pre_condition.as_ref() {
                if !pre_condition
                    .op
                    .check(self.variables.get(&pre_condition.name), pre_condition.value)
                {
                    self.current_prompt += 1;
                    continue;
                }
            }

            break;
        }

        self.get_current_prompt()
    }

    /// Gets a shared reference to the variables of this script.
    #[inline]
    pub fn variables(&self) -> &Variables {
        &self.variables
    }

    /// Returns an exclusive reference to the variables of this script.
    #[inline]
    pub fn variables_mut(&mut self) -> &mut Variables {
        &mut self.variables
    }
}

impl From<Story> for StoryExecutor {
    fn from(s: Story) -> Self {
        let mut variables = Variables::default();

        for action in &s.actions {
            let val = variables.get_mut(&action.name);
            action.op.execute(val, action.value);
        }

        Self {
            story: s,
            current_batch: 0,
            current_prompt: 0,
            variables: Variables::default(),
        }
    }
}
