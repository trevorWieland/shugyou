use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t = 10)]
    episodes: u64,
    #[arg(long, default_value_t = 120)]
    ticks: u64,
    #[arg(long, default_value_t = 42)]
    seed: u64,
}

fn main() {
    let args = Args::parse();
    let mut aggregate_tick = 0_u64;
    for episode in 0..args.episodes {
        let observation = game_simulation::run_headless_ticks(args.ticks, args.seed + episode);
        aggregate_tick += observation.tick;
    }
    println!(
        "balance scaffold complete: episodes={} aggregate_tick={}",
        args.episodes, aggregate_tick
    );
}
