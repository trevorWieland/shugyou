use bevy::prelude::*;
use game_observation::Observation;

#[derive(Resource)]
struct BootObservation(Observation);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.11)))
        .insert_resource(BootObservation(Observation::loaded(0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "shugyou - loaded".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, startup)
        .run();
}

fn startup(observation: Res<BootObservation>) {
    let status = game_presentation::status_from_observation(&observation.0);
    println!("client boot status: {status}");
    info!("client boot status: {status}");
}
