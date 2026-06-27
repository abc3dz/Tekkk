use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(FpsLimiter {
            last_frame: Instant::now(),
        })
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup_fps_timer)
        .add_systems(Update, print_fps)
        .add_systems(Update, limit_fps);
    }
}

#[derive(Resource)]
struct FpsLimiter {
    last_frame: Instant,
}

fn limit_fps(mut limiter: ResMut<FpsLimiter>) {
    let target = Duration::from_secs_f64(1.0 / 30.0);
    let elapsed = limiter.last_frame.elapsed();

    if elapsed < target {
        sleep(target - elapsed);
    }

    limiter.last_frame = Instant::now();
}

#[derive(Resource)]
struct FpsPrintTimer(Timer);

fn setup_fps_timer(mut commands: Commands) {
    commands.insert_resource(FpsPrintTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}

fn print_fps(
    time: Res<Time>,
    mut timer: ResMut<FpsPrintTimer>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            println!("FPS: {}", fps);
        } else {
            println!("FPS: waiting...");
        }
    }
}