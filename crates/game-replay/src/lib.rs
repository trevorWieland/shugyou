use game_command::GameCommand;
use game_observation::Observation;
use serde::{Deserialize, Serialize};

pub const REPLAY_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFrame {
    pub schema_version: u16,
    pub tick: u64,
    pub command: GameCommand,
    pub observation: Observation,
}

impl ReplayFrame {
    #[must_use]
    pub fn new(tick: u64, command: GameCommand, observation: Observation) -> Self {
        Self {
            schema_version: REPLAY_SCHEMA_VERSION,
            tick,
            command,
            observation,
        }
    }
}
