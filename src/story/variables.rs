use std::collections::HashMap;

/// A collection of variables available for story scripting.
#[derive(Debug, Clone, Default)]
pub struct Variables {
    variables: HashMap<String, i64>,
}

impl Variables {
    /// Gets the value of a specific variable.
    pub fn get(&self, name: &str) -> i64 {
        self.variables.get(name).copied().unwrap_or_default()
    }

    /// Gets an exclusive reference to a specific variable.
    pub fn get_mut(&mut self, name: &str) -> &mut i64 {
        self.variables.entry(name.into()).or_default()
    }
}
