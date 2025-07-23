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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Down,
    Up,
}

pub type ButtonStates = [ButtonState; 3];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Input {
    states: ButtonStates,
    last_state: ButtonStates,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            states: [ButtonState::Up; 3],
            last_state: [ButtonState::Up; 3],
        }
    }
}

impl Input {
    pub fn new(states: ButtonStates) -> Self {
        Self {
            states: states,
            last_state: states,
        }
    }

    pub fn button_state(&self, button: Button) -> ButtonState {
        self.states[button.index()]
    }

    pub fn pressed(&self, button: Button) -> bool {
        return self.states[button.index()] == ButtonState::Down
            && self.last_state[button.index()] == ButtonState::Up;
    }

    pub fn released(&self, button: Button) -> bool {
        return self.states[button.index()] == ButtonState::Up
            && self.last_state[button.index()] == ButtonState::Down;
    }

    pub fn update_state(&mut self, states: ButtonStates) {
        self.last_state = self.states;
        self.states = states;
    }
}
