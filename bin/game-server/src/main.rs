use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t = 32)]
    vector: u16,
}

fn main() {
    let args = Args::parse();
    let observation = game_simulation::run_headless_ticks(1, 1);
    println!(
        "game-server scaffold online; vector={} tick={} transport={:?}",
        args.vector,
        observation.tick,
        game_net::TransportMode::Local
    );
}
