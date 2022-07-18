use crate::car::*;
use bevy::prelude::*;
use bevy::{diagnostic::Diagnostics, diagnostic::FrameTimeDiagnosticsPlugin};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct MpsText;

#[derive(Component)]
pub struct KmphText;

#[derive(Component)]
pub struct WheelsWText;

#[derive(Component)]
pub struct WheelsTorqueText;

pub fn dash_fps_start_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bold: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let medium: Handle<Font> = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(2.0),
                    left: Val::Px(2.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: bold.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: medium.clone(),
                            font_size: 16.0,
                            color: Color::GOLD,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(FpsText);
}

pub fn dash_fps_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{:.1}", average);
            }
        }
    }
}

pub fn dash_speed_start_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    let bold: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let medium: Handle<Font> = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: medium.clone(),
                            font_size: 16.0,
                            color: Color::GOLD,
                        },
                    },
                    TextSection {
                        value: "m/s".to_string(),
                        style: TextStyle {
                            font: bold.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(MpsText);
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(25.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: medium.clone(),
                            font_size: 16.0,
                            color: Color::GOLD,
                        },
                    },
                    TextSection {
                        value: "km/h".to_string(),
                        style: TextStyle {
                            font: bold.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(KmphText);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(50.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: medium.clone(),
                            font_size: 16.0,
                            color: Color::GOLD,
                        },
                    },
                    TextSection {
                        value: "w".to_string(),
                        style: TextStyle {
                            font: bold.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(WheelsWText);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(70.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: medium.clone(),
                            font_size: 16.0,
                            color: Color::GOLD,
                        },
                    },
                    TextSection {
                        value: "t".to_string(),
                        style: TextStyle {
                            font: bold.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(WheelsTorqueText);
}

pub fn dash_speed_update_system(
    mut texts: ParamSet<(
        Query<&mut Text, With<MpsText>>,
        Query<&mut Text, With<KmphText>>,
        Query<&mut Text, With<WheelsWText>>,
        Query<&mut Text, With<WheelsTorqueText>>,
    )>,
    mut cars: Query<(&Velocity, With<HID>)>,
    mut wheels: Query<(&Velocity, &ExternalForce, With<Wheel>)>,
) {
    let (velocity, _) = cars.single_mut();
    let mps = velocity.linvel.length();
    texts.p0().single_mut().sections[0].value = format!("{:.1}", mps);

    let kmph = mps * 3.6;
    texts.p1().single_mut().sections[0].value = format!("{:.1}", kmph);

    let mut v_msg: String = "".to_string();
    let mut f_msg: String = "".to_string();
    for (v, f, _wheel) in wheels.iter_mut() {
        let v_s = format!("{:.1} ", v.angvel.length());
        v_msg = v_msg + &v_s;
        let f_s = format!("{:.1} ", f.torque.length());
        f_msg = f_msg + &f_s;
    }
    texts.p2().single_mut().sections[0].value = v_msg;
    texts.p3().single_mut().sections[0].value = f_msg;
}
