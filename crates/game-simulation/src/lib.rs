use bevy_ecs::prelude::Resource;
use game_command::GameCommand;
use game_content::ContentManifest;
use game_observation::Observation;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub struct SimulationTick(pub u64);

#[derive(Debug, Clone, Default, Resource)]
pub struct CommandBuffer(pub Vec<GameCommand>);

#[derive(Debug, Resource)]
pub struct SimulationRng(pub ChaCha8Rng);

impl SimulationRng {
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        let mut bytes = [0_u8; 32];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        Self(ChaCha8Rng::from_seed(bytes))
    }
}

#[derive(Debug, Clone)]
pub struct SimulationState {
    pub tick: SimulationTick,
    pub commands_applied: u64,
}

impl SimulationState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick: SimulationTick(0),
            commands_applied: 0,
        }
    }

    pub fn step(&mut self, command_buffer: &mut CommandBuffer, _rng: &mut SimulationRng) {
        self.commands_applied += command_buffer.0.len() as u64;
        command_buffer.0.clear();
        self.tick.0 += 1;
    }
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn run_headless_ticks(ticks: u64, seed: u64) -> Observation {
    let _content_manifest = ContentManifest::default();
    let mut state = SimulationState::new();
    let mut commands = CommandBuffer(vec![GameCommand::noop()]);
    let mut rng = SimulationRng::from_seed(seed);

    for _ in 0..ticks {
        state.step(&mut commands, &mut rng);
        commands.0.push(GameCommand::noop());
    }

    Observation::loaded(state.tick.0)
}
