use serde::{Deserialize, Serialize};

pub const OBSERVATION_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u16,
    pub tick: u64,
    pub loaded: bool,
}

impl Observation {
    #[must_use]
    pub fn loaded(tick: u64) -> Self {
        Self {
            schema_version: OBSERVATION_SCHEMA_VERSION,
            tick,
            loaded: true,
        }
    }
}
