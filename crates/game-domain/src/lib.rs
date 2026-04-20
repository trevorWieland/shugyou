use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DOMAIN_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid domain state")]
    InvalidState,
}
