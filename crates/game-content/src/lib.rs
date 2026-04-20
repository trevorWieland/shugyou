use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentManifest {
    pub version: u16,
    pub profile: String,
}

impl Default for ContentManifest {
    fn default() -> Self {
        Self {
            version: 1,
            profile: "day0".to_owned(),
        }
    }
}
