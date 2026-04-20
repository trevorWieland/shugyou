use game_domain::EntityId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const COMMAND_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameCommand {
    Noop {
        schema_version: u16,
    },
    Move {
        schema_version: u16,
        target: EntityId,
        dx: i32,
        dy: i32,
    },
}

impl GameCommand {
    #[must_use]
    pub fn noop() -> Self {
        Self::Noop {
            schema_version: COMMAND_SCHEMA_VERSION,
        }
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("unsupported command schema version: {0}")]
    UnsupportedSchemaVersion(u16),
}

pub fn validate(command: &GameCommand) -> Result<(), CommandError> {
    let version = match command {
        GameCommand::Noop { schema_version } | GameCommand::Move { schema_version, .. } => {
            *schema_version
        }
    };
    if version != COMMAND_SCHEMA_VERSION {
        return Err(CommandError::UnsupportedSchemaVersion(version));
    }
    Ok(())
}
