use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t = 60)]
    ticks: u64,
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

fn main() {
    let args = Args::parse();
    let observation = game_simulation::run_headless_ticks(args.ticks, args.seed);
    println!(
        "headless simulation complete: ticks={} loaded={} schema_version={}",
        observation.tick, observation.loaded, observation.schema_version
    );
}
