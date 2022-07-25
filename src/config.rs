use bevy::prelude::*;
use std::f32::consts::PI;

pub struct Config {
    pub translation: Vec3,
    pub quat: Quat,
    pub cars_count: i16,
    pub use_brain: bool,
    pub friction: f32,
    pub restitution: f32,
    pub max_torque: f32,
    pub hid_car: Option<Entity>,
    pub camera_follow: Option<Entity>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cars_count: 10,
            use_brain: true,
            max_torque: 400.,
            translation: Vec3::new(0., 1., 0.),
            quat: Quat::from_rotation_y(-PI * 0.2),
            restitution: 0.000_000_000_000_000_001,
            friction: 10.,
            hid_car: None,
            camera_follow: None,
        }
    }
}