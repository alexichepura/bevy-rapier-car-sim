use bevy::app::EventReader;
use bevy::ecs::system::ResMut;
use bevy::input::gamepad::{Gamepad, GamepadEvent, GamepadEventType};
use bevy::utils::HashSet;

#[derive(Default)]
pub struct GamepadLobby {
    pub gamepads: HashSet<Gamepad>,
}

pub fn gamepad_lobby_system(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                lobby.gamepads.insert(*gamepad);
                println!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                lobby.gamepads.remove(gamepad);
                println!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}
