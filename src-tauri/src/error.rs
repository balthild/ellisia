use std::fmt::Display;

use serde::{Serialize, Serializer};

#[derive(Debug)]
/// A serializable error type. Used in the return type of tauri::command handlers.
pub struct CommandError(pub String);

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        #[cfg(debug_assertions)]
        return CommandError(format!("{:?}", err));
        #[cfg(not(debug_assertions))]
        return CommandError(err.to_string());
    }
}

impl std::error::Error for CommandError {}
