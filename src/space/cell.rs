const ALIVE_STATE: u8 = 255;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Cell {
    pub x: u16,
    pub y: u16,
    state: u8,
}

#[allow(dead_code)]
impl Cell {
    pub fn new(x: u16, y: u16, state: u8) -> Cell {
        Cell {
            x, y,
            state
        }
    }

    pub fn get_state(&self) -> u8 {
        self.state
    }

    pub fn set_state(&mut self, state: u8) {
        self.state = state;
    }

    pub fn revive(&mut self) {
        self.state = ALIVE_STATE;
    }

    pub fn kill(&mut self) {
        self.state = 0;
    }

    pub fn age(&mut self) {
        if self.get_state() > 0 {
            self.set_state(self.get_state() - 17);
        }
    }

    pub fn is_alive(&self) -> bool {
        self.state == ALIVE_STATE
    }

}
