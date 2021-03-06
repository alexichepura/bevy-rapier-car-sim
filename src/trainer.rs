use crate::{brain::*, car::Car, config::Config, progress::*};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{ExternalForce, Velocity};
use std::{cmp::Ordering, fs};

const PAUSE: f64 = 3.;
const LINVEL_FORCE: f32 = 10000.;
const ANGVEL_FORCE: f32 = 2.;

pub struct Trainer {
    pub interval: f64,
    pub generation: i32,
    pub record: f32,
    pub last_check_at: f64,
    pub best_brain: Option<CarBrain>,
}

impl Default for Trainer {
    fn default() -> Self {
        Self {
            interval: 20.,
            generation: 0,
            record: 0.,
            last_check_at: 0.,
            best_brain: None,
        }
    }
}

#[derive(Component)]
pub struct TrainerTimingText;
#[derive(Component)]
pub struct TrainerRecordDistanceText;
#[derive(Component)]
pub struct TrainerGenerationText;

pub fn trainer_system(
    mut config: ResMut<Config>,
    mut trainer: ResMut<Trainer>,
    time: Res<Time>,
    mut cars: Query<
        (
            &mut CarProgress,
            &mut CarBrain,
            &mut Transform,
            &mut Car,
            &mut ExternalForce,
        ),
        With<CarProgress>,
    >,
    mut dash_set: ParamSet<(
        Query<&mut Text, With<TrainerTimingText>>,
        Query<&mut Text, With<TrainerRecordDistanceText>>,
        Query<&mut Text, With<TrainerGenerationText>>,
    )>,
) {
    let seconds = time.seconds_since_startup();
    if config.reset_pause_until > seconds {
        return;
    }
    if config.reset_pause_until > 0. {
        config.reset_pause_until = 0.;
        config.use_brain = true;
        for (_, _, _, mut car, mut f) in cars.iter_mut() {
            car.use_brain = true;
            *f = ExternalForce::default();
        }
    }
    if !config.use_brain {
        return;
    }
    let seconds_diff = seconds - trainer.last_check_at;

    let mut q_trainer_timing = dash_set.p0();
    let mut text = q_trainer_timing.single_mut();
    let round_seconds = ((trainer.interval - seconds_diff) * 10.).round() / 10.;
    text.sections[1].value = round_seconds.to_string();

    if seconds_diff > trainer.interval {
        trainer.last_check_at = seconds;

        let best_car = cars
            .iter()
            .max_by(|a, b| {
                if a.0.meters > b.0.meters {
                    return Ordering::Greater;
                }
                Ordering::Less
            })
            .unwrap();
        let (progress, best_brain, _, _, _) = best_car;
        trainer.best_brain = Some(best_brain.clone());
        let best_brain = best_brain.clone();

        let minimal_progress_delta = 1.;
        if progress.meters > (trainer.record + minimal_progress_delta) {
            println!("distance record {:.1}", progress.meters);
            trainer.record = progress.meters;
        } else {
            trainer.generation += 1;
            trainer.record = 0.;
            config.use_brain = false;
            config.reset_pause_until = time.seconds_since_startup() + 5.;
            for (_i, (_progress, mut brain, mut t, mut car, mut f)) in cars.iter_mut().enumerate() {
                let cloned_best: CarBrain = CarBrain::clone_randomised(&best_brain);
                brain.levels = cloned_best.levels.clone();
                car.gas = 0.;
                car.brake = 0.;
                car.steering = 0.;
                car.use_brain = false;
                *t = car.init_transform;
                *f = ExternalForce::default()
            }
            println!("new generation {:?}", trainer.generation);

            let mut brain_dump = best_brain.clone();
            for level in brain_dump.levels.iter_mut() {
                level.inputs.fill(0.);
                level.outputs.fill(0.);
            }
            let serialized = serde_json::to_string(&brain_dump).unwrap();
            println!("saving brain.json");
            fs::write("brain.json", serialized).expect("Unable to write brain.json");
        }
    }

    let mut q_record_distance_text = dash_set.p1();
    let mut record_text = q_record_distance_text.single_mut();
    record_text.sections[1].value = ((trainer.record * 10.).round() / 10.).to_string();

    let mut q_generation_text = dash_set.p2();
    let mut generation_text = q_generation_text.single_mut();
    generation_text.sections[1].value = trainer.generation.to_string();
}

pub fn reset_pos_system(
    config: Res<Config>,
    time: Res<Time>,
    mut q_car: Query<(&mut Transform, &mut Car, &mut ExternalForce, &Velocity)>,
) {
    let seconds = time.seconds_since_startup();
    for (mut t, mut car, mut f, v) in q_car.iter_mut() {
        if t.translation.y > 500. || t.translation.y < 0.
        // || v.linvel.length() > 100.
        // || v.angvel.length() > PI
        {
            println!("car is out of bound {:?}", t.translation.round());
            car.gas = 0.;
            car.brake = 0.;
            car.steering = 0.;
            car.use_brain = false;
            car.reset_pause_until = seconds + PAUSE;
            *t = car.init_transform;
            *f = ExternalForce::default()
        }
        if car.reset_pause_until > seconds {
            *t = car.init_transform;
            f.force = -v.linvel * LINVEL_FORCE;
            f.torque = -v.angvel * ANGVEL_FORCE;
        } else if car.reset_pause_until > 0. {
            *t = car.init_transform;
            *f = ExternalForce::default();
            car.use_brain = config.use_brain;
            car.reset_pause_until = 0.;
        }
    }
}

pub fn reset_spawn_key_system(
    keys: Res<Input<KeyCode>>,
    mut config: ResMut<Config>,
    time: Res<Time>,
    mut query: Query<(&mut Car, &mut Transform)>,
) {
    if keys.just_pressed(KeyCode::Space) {
        println!("KeyCode::Space, cleanup");
        config.use_brain = false;
        config.reset_pause_until = time.seconds_since_startup() + PAUSE;
        for (mut car, mut t) in query.iter_mut() {
            car.gas = 0.;
            car.brake = 0.;
            car.steering = 0.;
            car.use_brain = false;
            *t = car.init_transform;
        }
    }
}
pub fn reset_force_system(
    config: Res<Config>,
    time: Res<Time>,
    mut q_carforces: Query<(&Velocity, &mut ExternalForce, &mut Transform, &Car), With<Car>>,
) {
    if config.reset_pause_until > time.seconds_since_startup() {
        for (v, mut f, mut t, car) in q_carforces.iter_mut() {
            f.force = -v.linvel * LINVEL_FORCE;
            f.torque = -v.angvel * ANGVEL_FORCE;
            *t = car.init_transform;
        }
    }
}

// TODO velocity does not work
// pub fn reset_spawn_key_system(keys: Res<Input<KeyCode>>, mut q: Query<&mut Velocity>) {
//     if keys.just_pressed(KeyCode::Space) {
//         println!("KeyCode::Space, cleanup");
//         for mut v in &mut q {
//             println!("reset velocity");
//             *v = Velocity::zero();
//         }
//     }
// }

// TODO impulse does not work
// pub fn reset_spawn_key_system(
//     keys: Res<Input<KeyCode>>,
//     mut q: Query<&mut ExternalImpulse, &Wheel>,
// ) {
//     if keys.just_pressed(KeyCode::Space) {
//         println!("KeyCode::Space, cleanup");
//         for mut impulse in &mut q {
//             println!("impulse.impulse = Vec3::Y * 1_000_000.;");
//             impulse.impulse = Vec3::X * 1_000_000.;
//         }
//     }
// }

// https://github.com/dimforge/bevy_rapier/issues/196
// pub fn reset_spawn_key_system(
//     keys: Res<Input<KeyCode>>,
//     mut commands: Commands,
//     query: Query<Entity, With<MultibodyJoint>>,
// ) {
//     if !keys.just_pressed(KeyCode::Space) {
//         return;
//     }
//     println!("KeyCode::Space, cleanup");
//     for e in query.iter() {
//         println!("cleanup MultibodyJoint");
//         /// commands.entity(e).remove::<MultibodyJoint>();
//         commands.entity(e).despawn();
//     }
// }
