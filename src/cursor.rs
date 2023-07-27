use crate::min_todo::InputMode;

//Represent current cursor position (0,0 based)
#[derive(Debug)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl Cursor {
    pub fn render(&self, mode: &InputMode) {
        print!(
            "{}{}{}",
            termion::cursor::Show,
            termion::cursor::Goto(self.x as u16 + 1, self.y as u16 + 1), //We have to add 1, bcs
            //GoTo() is 1,1 based, but
            //we represent the coords
            //as 0,0 based
            match mode {
                InputMode::Normal => termion::cursor::SteadyBlock.to_string(),
                InputMode::Insert => termion::cursor::SteadyBar.to_string(),
            },
        );
    }

    pub fn move_x(&mut self, x: i32) {
        if x.is_negative() {
            if self.x > 0 {
                self.x -= x.wrapping_abs() as usize;
            }
        } else {
            self.x += x.wrapping_abs() as usize;
        }
    }
    pub fn move_y(&mut self, y: i32) {
        if y.is_negative() {
            if self.y > 0 {
                self.y -= y.wrapping_abs() as usize;
            }
        } else {
            self.y += y.wrapping_abs() as usize;
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}
