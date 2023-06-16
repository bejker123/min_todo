use std::{
    error::Error,
    io::{self, Read, Write},
};

use termion::raw::IntoRawMode;

pub struct Line {
    pub content: String,
}

pub struct Block {
    title: String,
    width: usize,
    height: usize,
    lines: Vec<Line>,
}

impl Line {
    pub fn width(&self) -> usize {
        self.content.len()
    }

    pub fn render(&self, width: usize) -> Result<(), Box<dyn Error>> {
        let width = width.min(self.width());
        print!("{}\n\r", &self.content[..width]);
        Ok(())
    }
}

impl Block {
    pub fn render(&self) -> Result<(), Box<dyn Error>> {
        print!("{}\n\r", self.title);

        let height = self.height.min(self.lines.len());
        for line in &self.lines[..height] {
            line.render(self.width)?;
        }
        Ok(())
    }

    pub fn new(title: String, width: usize, height: usize, lines: Vec<Line>) -> Self {
        Self {
            title,
            width,
            height,
            lines,
        }
    }
    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }
}

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

pub struct Renderer {
    content: Vec<Block>,
    cursor: Cursor,
    changed: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            cursor: Cursor::default(),
            changed: true,
        }
    }

    pub fn add_block(&mut self, block: Block) {
        self.content.push(block);
    }

    //Clear and move the cursor to (1,1)
    fn clear() {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }

    fn flush() -> Result<(), Box<dyn Error>> {
        std::io::stdout().flush()?;

        Ok(())
    }

    fn exit() -> ! {
        Self::clear();
        std::process::exit(0)
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        // let mut buf = Vec::new();
        // std::io::stdin().(&mut buf)?;
        // println!("{:?}", buf);
        let mut buffer = [0];
        let stdout = io::stdout().into_raw_mode().unwrap();
        let mut stdin = io::stdin();

        stdout.lock().flush().unwrap();

        stdin.read_exact(&mut buffer)?;
        // println!("{:?}", buffer);
        match buffer.get(0) {
            Some(o) => match o {
                3 => Self::exit(),
                106 => self.cursor.move_y(1),
                107 => self.cursor.move_y(-1),
                104 => self.cursor.move_x(-1),
                108 => self.cursor.move_x(1),
                113 => Self::exit(),
                _ => {}
            },
            None => {}
        }
        // println!("{buffer:?}");
        self.changed = true;
        // std::thread::sleep_ms(100);
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.changed {
            return Ok(());
        }
        self.changed = false;
        Self::clear();
        for c in &self.content {
            c.render()?;
        }
        Self::flush()?;
        self.cursor.render()?;
        Ok(())
    }
}
