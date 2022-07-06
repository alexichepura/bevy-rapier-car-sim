use crate::Car;
use bevy::prelude::*;
use core::f32::consts::PI;
use smooth_bevy_cameras::controllers::unreal::{UnrealCameraBundle, UnrealCameraController};

use bevy::render::camera::Camera3d;

pub fn camera_focus_system(
    mut transforms: ParamSet<(
        Query<(&mut Transform, &Camera, With<Camera3d>)>,
        Query<(&Transform, &Car)>,
    )>,
) {
    let p1 = transforms.p1();
    let (car_transform, _car) = p1.single();
    let mut tf = Transform::from_matrix(car_transform.compute_matrix());
    let shift_vec: Vec3 = tf.rotation.mul_vec3(Vec3::new(0., 2.5, -8.));
    tf.translation.x = tf.translation.x + shift_vec.x;
    tf.translation.y = tf.translation.y + shift_vec.y;
    tf.translation.z = tf.translation.z + shift_vec.z;
    tf.rotate(Quat::from_rotation_y(-PI));
    tf.look_at(car_transform.translation + Vec3::new(0., 1., 0.), Vec3::Y);
    for (mut cam_transform, _, _) in transforms.p0().iter_mut() {
        *cam_transform = tf;
    }
}

pub fn camera_system(mut commands: Commands) {
    // commands.spawn_bundle(PerspectiveCameraBundle {
    //     transform: Transform::from_translation(Vec3::new(10., 2.5, 10.))
    //         .looking_at(Vec3::ZERO, Vec3::Y),
    //     ..Default::default()
    // });
    commands
        .spawn_bundle(PerspectiveCameraBundle::default())
        .insert_bundle(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            Vec3::new(0., 5., 10.),
            Vec3::new(0., 0., 0.),
        ));
    commands.spawn_bundle(UiCameraBundle::default());
}