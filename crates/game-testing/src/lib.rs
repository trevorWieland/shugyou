#[must_use]
pub fn deterministic_smoke(seed: u64, ticks: u64) -> bool {
    let left = game_simulation::run_headless_ticks(ticks, seed);
    let right = game_simulation::run_headless_ticks(ticks, seed);
    left == right
}

#[cfg(test)]
mod tests {
    #[test]
    fn headless_simulation_is_deterministic_for_same_seed() {
        assert!(crate::deterministic_smoke(42, 60));
    }
}
