use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A function that may be executed on a variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    /// Sets the variable to a specific value.
    Set,
    /// Adds a specific value to the variable.
    Add,
    /// Subtracts a specific value from the variable.
    Sub,
}

impl Operation {
    /// Executes the operation on the provided value
    pub fn execute(self, value: &mut i64, other: i64) {
        match self {
            Self::Set => *value = other,
            Self::Add => *value += other,
            Self::Sub => *value -= other,
        }
    }
}

/// A comparaison function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Compare {
    /// The variable must equal a specific value.
    Equal,
    /// The variable must not equal a specific value.
    Not,
    /// The variable must be less than a specific value.
    Less,
    /// The variable must be greater than a specfic value.
    More,
}

impl Compare {
    /// Determines whether `value` `op` `other`.
    pub fn check(self, value: i64, other: i64) -> bool {
        match self {
            Self::Equal => value == other,
            Self::Not => value != other,
            Self::Less => value < other,
            Self::More => value > other,
        }
    }
}

/// A pre-condition for a specific [`Prompt`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    /// The name of the variable that's being checked.
    pub name: String,
    /// The comparaison function.
    pub op: Compare,
    /// The value against which the variable is beging checked.
    pub value: i64,
}

/// An action that may be taken when the player chooses a specific answer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// The name of the variable that'll be modified.
    pub name: String,
    /// The operation that'll be executed.
    pub op: Operation,
    /// The other parameter of the operation.
    pub value: i64,
}

/// An possible answer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Answer {
    /// The text of the answer.
    #[serde(default)]
    pub text: String,
    /// A collection of actions for this answer.
    #[serde(default)]
    pub actions: Vec<Action>,
}

/// A prompt that may be presented to the player.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prompt {
    /// A pre-condition for this prompt. If the condition is evaluated to `false`, then this prompt
    /// is skipped. If no condition is specified, the prompt is presented.
    #[serde(rename = "if")]
    pub pre_condition: Option<Condition>,
    /// The request string that'll be animated on the terminal.
    pub request: String,
    /// The possible answers for this prompt.
    pub answers: Vec<Answer>,
}

/// A batch of prompts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Batch {
    /// Whether the prompts of this [`Batch`] can be randomized, or, on the contrary, whether they
    /// should be present in a fixed order.
    #[serde(rename = "random")]
    pub randomized: bool,
    /// The prompts that are part of this [`Batch`].
    pub prompts: Vec<Prompt>,
}

/// The main story structure. This basically acts as a collection of [`Batch`]es.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Story {
    /// A collection of actions that should be taken at the begining of the game.
    pub actions: Vec<Action>,
    /// The batches that are to be presented to the player before ending the game.
    pub batches: Vec<Batch>,
}

/// Parses a [`Story`] instance at the provided path.
pub fn parse_story(path: &Path) -> io::Result<Story> {
    let file = BufReader::new(File::open(path)?);
    let story = serde_json::from_reader(file)?;
    Ok(story)
}
