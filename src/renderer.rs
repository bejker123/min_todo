const START_SCROLL_UP: i32 = 5;

use std::{
    error::Error,
    io::{self, Read, Write},
};

#[derive(Debug)]
pub struct Line {
    pub content: String,
    pub width: usize,
}

impl Line {
    pub fn new<T: std::fmt::Display>(content: T) -> Self {
        Self {
            content: content.to_string(),
            width: content.to_string().len(),
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn render(&self) {
        let width = self.width.min(self.len());
        print!("\n\r{}", &self.content[..width]);
    }
}

#[derive(Debug)]
pub struct Cursor {
    x: i32,
    y: i32,
}

impl Cursor {
    pub fn render(&self) -> Result<(), Box<dyn Error>> {
        print!(
            "{esc}[{};{}H{esc}[25h",
            self.y + 1,
            self.x + 1,
            esc = 27 as char
        );
        Renderer::flush()?;
        Ok(())
    }

    pub fn move_x(&mut self, x: i32) {
        self.x += x;
        if self.x < 0 {
            self.x = 0;
        }
    }
    pub fn move_y(&mut self, y: i32) {
        self.y += y;
        if self.y < 0 {
            self.y = 0;
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug)]
pub struct Renderer {
    content: Vec<Line>,
    cursor: Cursor,
    changed: bool,
    scroll_beg: usize,
    scroll_end: usize,
    start_scroll_down: i32,
}

impl Renderer {
    pub fn new() -> Self {
        let term_columns = termion::terminal_size().unwrap().1;
        Self {
            content: Vec::new(),
            cursor: Cursor::default(),
            changed: true,
            scroll_beg: 0,
            scroll_end: term_columns as usize,
            start_scroll_down: term_columns as i32 - 5,
        }
    }

    pub fn add_line(&mut self, line: Line) {
        self.content.push(line);
        // self.scroll_end += 1;
    }

    //Clear and move the cursor to (1,1)
    fn clear() {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }

    fn flush() -> Result<(), Box<dyn Error>> {
        std::io::stdout().flush()?;

        Ok(())
    }

    fn exit() -> bool {
        Self::clear();
        let _ = Self::flush(); //Here we don't care if we succed or not.
        false
    }
    /*
     *
     *
     *
     */
    fn move_cur_down(&mut self) {
        if self.cursor.y >= self.start_scroll_down {
            if self.scroll_beg < self.scroll_end {
                self.scroll_beg += 1;
            }
            // if self.scroll_end < self.content.len() {
            self.scroll_end += 1;
            // }
        } else {
            self.cursor.move_y(1);
        }
    }
    fn move_cur_up(&mut self) {
        if self.cursor.y == START_SCROLL_UP && self.scroll_beg > 0_usize {
            self.scroll_beg -= 1;
            if self.scroll_end > 0 {
                self.scroll_end -= 1;
            }
        } else {
            self.cursor.move_y(-1);
        }
    }

    pub fn handle_command(&mut self, c: char) -> bool {
        match c {
            //Ctrl-C
            '\u{3}' | 'q' => return Self::exit(),
            'j' => self.move_cur_down(),
            'k' => self.move_cur_up(),
            'h' => self.cursor.move_x(-1),
            'l' => self.cursor.move_x(1),
            _ => {}
        }
        true
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        // let mut buf = Vec::new();
        // std::io::stdin().(&mut buf)?;
        // println!("{:?}", buf);
        let mut buffer = [0];
        let mut stdin = io::stdin();

        // stdout.lock().flush().unwrap();

        stdin.read_exact(&mut buffer)?;
        // println!("{:?}", buffer);
        if let Some(c) = buffer.first() {
            if !self.handle_command(*c as char) {
                return Ok(false);
            }
        }
        // println!("{buffer:?}");
        self.changed = true;
        // std::thread::sleep_ms(100);
        Ok(true)
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.changed {
            return Ok(());
        }
        self.changed = false;
        Self::clear();

        let start = self.scroll_beg;
        // .min(self.scroll_end - START_SCROLL_UP as usize);
        let end = self.scroll_end.min(self.content.len());

        for c in &self.content[start..end] {
            c.render();
        }
        Self::flush()?;
        self.cursor.render()?;
        Ok(())
    }
}
