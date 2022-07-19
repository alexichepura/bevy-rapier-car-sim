use std::fs;

use crate::car::*;
use crate::track::STATIC_GROUP;
use bevy::prelude::*;
use bevy_mod_picking::PickingEvent;
use bevy_polyline::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
use rand::{distributions::Standard, Rng};
use serde::{Deserialize, Serialize};

fn car_lerp(a: f32, random_0_to_1: f32) -> f32 {
    let b = random_0_to_1 * 2. - 1.;
    let t = 0.1;
    a + (b - a) * t
}

#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct CarBrain {
    levels: Vec<Level>,
}
impl CarBrain {
    pub fn new() -> CarBrain {
        let ins = Level::new(5, 6);
        let hidden = Level::new(6, 4);
        CarBrain {
            levels: [ins, hidden].to_vec(),
        }
    }
    pub fn feed_forward(&mut self, new_inputs: Vec<f32>) {
        let mut outputs: Vec<f32> = new_inputs.clone();
        for level in self.levels.iter_mut() {
            level.feed_forward(outputs.clone());
            outputs = level.outputs.clone();
        }
    }

    pub fn mutate_random(&mut self) {
        let mut rng = rand::thread_rng();
        for level in self.levels.iter_mut() {
            for bias in level.biases.iter_mut() {
                *bias = car_lerp(*bias, rng.gen::<f32>());
            }
            for weighti in level.weights.iter_mut() {
                for weight in weighti.iter_mut() {
                    *weight = car_lerp(*weight, rng.gen::<f32>());
                }
            }
        }
    }

    pub fn clone_randomised(brain: Option<CarBrain>) -> Option<CarBrain> {
        if let Some(brain) = brain {
            let mut rng = rand::thread_rng();
            let mut levels: Vec<Level> = vec![];
            for level in brain.levels.iter() {
                let mut cloned_level = level.clone();
                for bias in cloned_level.biases.iter_mut() {
                    *bias = car_lerp(*bias, rng.gen::<f32>());
                }
                for weighti in cloned_level.weights.iter_mut() {
                    for weight in weighti.iter_mut() {
                        *weight = car_lerp(*weight, rng.gen::<f32>());
                    }
                }
                levels.push(cloned_level)
            }
            return Some(CarBrain { levels });
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Level {
    inputs: Vec<f32>,
    outputs: Vec<f32>,
    weights: Vec<Vec<f32>>,
    biases: Vec<f32>,
}

impl Level {
    pub fn new(n_in: usize, n_out: usize) -> Level {
        let inputs: Vec<f32> = vec![0.; n_in];
        let outputs: Vec<f32> = vec![0.; n_out];
        let weights: Vec<Vec<f32>> = (0..n_in)
            .map(|_| thread_rng().sample_iter(Standard).take(n_out).collect())
            .collect();
        let biases: Vec<f32> = thread_rng().sample_iter(Standard).take(n_out).collect();

        Level {
            weights,
            biases,
            inputs,
            outputs,
        }
    }
    pub fn feed_forward(&mut self, new_inputs: Vec<f32>) {
        for (index, input) in self.inputs.iter_mut().enumerate() {
            *input = new_inputs[index];
        }
        for (index_out, output) in self.outputs.iter_mut().enumerate() {
            let mut sum: f32 = 0.;
            for (index_in, input) in self.inputs.iter_mut().enumerate() {
                sum = sum + *input * self.weights[index_in][index_out];
            }
            if sum > self.biases[index_out] {
                *output = 1.;
            } else {
                *output = 0.;
            }
        }
    }
}
#[derive(Component)]
pub struct CarSensor;

pub fn car_brain_system(
    rapier_context: Res<RapierContext>,
    mut cars: Query<(
        &mut Car,
        &Transform,
        &mut CarBrain,
        &Children,
        With<CarBrain>,
    )>,
    polylines: ResMut<Assets<Polyline>>,
    rays: Query<(Entity, &Handle<Polyline>)>,
) {
    for (mut car, transform, mut brain, children, _) in cars.iter_mut() {
        let mut inputs: Vec<f32> = Vec::new();
        let max_toi: f32 = 10.;

        for &child in children.iter() {
            if let Ok((_, polyline)) = rays.get(child) {
                let vertices = &polylines.get(polyline).unwrap().vertices;
                let ray_origin = transform.translation + transform.rotation.mul_vec3(vertices[0]);
                let ray_dir = transform.rotation.mul_vec3(vertices[1]);
                let hit = rapier_context.cast_ray(
                    ray_origin,
                    ray_dir,
                    max_toi,
                    false,
                    QueryFilter {
                        // CollisionGroups::new(STATIC_GROUP, u32::MAX)
                        // groups: Some(InteractionGroups::new(CAR_TRAINING_GROUP, STATIC_GROUP)),
                        // groups: Some(InteractionGroups::new(STATIC_GROUP, u32::MAX)),
                        // groups: Some(InteractionGroups::new(STATIC_GROUP, u32::MAX)),
                        ..default()
                    },
                    // QueryFilter::new(),
                    // QueryFilter::exclude_dynamic(),
                    // QueryFilter::only_fixed(),
                    // QueryFilter::from(CollisionGroups::new(CAR_TRAINING_GROUP, STATIC_GROUP)),
                    // QueryFilter::from(InteractionGroups::new(STATIC_GROUP, STATIC_GROUP)),
                );
                match hit {
                    Some((_, sensor_units)) => {
                        if sensor_units > 1. {
                            inputs.push(0.);
                            return;
                        }
                        inputs.push(sensor_units);
                    }
                    None => inputs.push(-1.),
                }
            } else {
                // println!("not a polyline");
            }
        }
        if inputs.len() != 5 {
            println!("inputs 5!={:?}", inputs);
            inputs = vec![0., 0., 0., 0., 0.];
        }

        if !car.use_brain {
            return;
        }

        brain.feed_forward(inputs.clone());

        let outputs: &Vec<f32> = &brain.levels.last().unwrap().outputs;

        let gas = outputs[0];
        let brake = outputs[1];
        let left = outputs[2];
        let right = outputs[3];

        car.gas = gas;
        car.brake = brake;
        car.steering = -left + right;
    }
}

pub fn cars_pick_brain_mutate_restart(
    mut events: EventReader<PickingEvent>,
    mut cars: Query<(&mut CarBrain, &mut Transform, With<CarBrain>)>,
    car_init: Res<CarInit>,
) {
    let mut selected_brain: Option<CarBrain> = None;
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(e) => {
                let (brain, _, _) = cars.get(*e).unwrap();
                selected_brain = Some(brain.clone());
            }
            _ => (),
        }
    }
    if let Some(selected_brain) = selected_brain {
        let serialized = serde_json::to_string(&selected_brain).unwrap();
        println!("saving brain.json");
        fs::write("brain.json", serialized).expect("Unable to write brain.json");

        for (mut brain, mut transform, _) in cars.iter_mut() {
            let mut new_brain = selected_brain.clone();
            new_brain.mutate_random();
            *brain = new_brain;
            *transform =
                Transform::from_translation(car_init.translation).with_rotation(car_init.quat);
        }
    }
}
