use game_command::GameCommand;

pub trait ModHook: Send + Sync {
    fn transform_command(&self, command: GameCommand) -> GameCommand;
}
