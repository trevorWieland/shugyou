use game_observation::Observation;

pub const LOADED_STATUS: &str = "loaded";

#[must_use]
pub fn status_from_observation(observation: &Observation) -> &'static str {
    if observation.loaded {
        LOADED_STATUS
    } else {
        "loading"
    }
}
