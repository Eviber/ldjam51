//! Defines the structures and logic required to parse the main story .JSON file.

use std::collections::HashMap;

mod model;
pub use self::model::*;

mod variables;
pub use self::variables::*;

mod executor;
pub use self::executor::*;
