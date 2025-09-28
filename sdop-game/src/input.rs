use crate::game_consts;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

impl Button {
    pub fn index(&self) -> usize {
        match self {
            Button::Left => 0,
            Button::Middle => 1,
            Button::Right => 2,
        }
    }
}

pub const ALL_BUTTONS: [Button; 3] = [Button::Left, Button::Middle, Button::Right];

pub fn random_button(rng: &mut fastrand::Rng) -> Button {
    match rng.usize(0..=2) {
        0 => Button::Left,
        1 => Button::Middle,
        2 => Button::Right,
        _ => unreachable!(),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Down,
    Up,
}

pub type ButtonStates = [ButtonState; 3];

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Input {
    temperature: f32,
    states: ButtonStates,
    last_state: ButtonStates,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            temperature: game_consts::ROOM_TEMPTURE,
            states: [ButtonState::Up; 3],
            last_state: [ButtonState::Up; 3],
        }
    }
}

impl Input {
    pub fn new(states: ButtonStates) -> Self {
        Self {
            temperature: game_consts::ROOM_TEMPTURE,
            states,
            last_state: states,
        }
    }

    pub fn button_state(&self, button: Button) -> ButtonState {
        self.states[button.index()]
    }

    pub fn any_pressed(&self) -> bool {
        self.pressed(Button::Left)
            || self.pressed(Button::Middle)
            || self.pressed(Button::Right)
    }

    pub fn pressed(&self, button: Button) -> bool {
        self.states[button.index()] == ButtonState::Down
            && self.last_state[button.index()] == ButtonState::Up
    }

    pub fn released(&self, button: Button) -> bool {
        self.states[button.index()] == ButtonState::Up
            && self.last_state[button.index()] == ButtonState::Down
    }

    pub fn update_state(&mut self, states: ButtonStates) {
        self.last_state = self.states;
        self.states = states;
    }

    pub fn update_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    pub fn temperature(&self) -> f32 {
        self.temperature
    }
}
