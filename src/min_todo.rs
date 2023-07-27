pub type Buffer = [u8; 4];

use crate::{
    char_parser,
    command_parser::{CommandParser, InsertModeCommand, NormalModeCommand},
    cursor::Cursor,
};
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
        // let width = self.width.min(self.len());
        print!("\n\r{}", self.content);
        // print!("\n\r{}", &self.content[..width]);
    }
}

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct MinTodo {
    content: Vec<Line>,
    cursor: Cursor,
    changed: bool,
    scroll_beg: usize,
    scroll_end: usize,
    start_scroll_up: usize,
    start_scroll_down: usize,
    term_columns: usize,
    command_parser: CommandParser,
    mode: InputMode,
    bottom_line: Option<Line>,
}

impl MinTodo {
    pub fn new() -> Self {
        let term_columns = termion::terminal_size().unwrap().1 as usize;
        Self {
            content: Vec::new(),
            cursor: Cursor::default(),
            changed: true,
            scroll_beg: 0,
            scroll_end: term_columns,
            term_columns,
            start_scroll_up: 5,
            start_scroll_down: term_columns - 5,
            command_parser: CommandParser::new(),
            mode: InputMode::Normal,
            bottom_line: None,
        }
    }

    pub fn add_line(&mut self, line: Line) {
        self.content.push(line);
        // self.scroll_end += 1;
    }

    //Clear and move the cursor to (1,1)
    fn clear() {
        print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
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

    fn move_cur_down(&mut self) {
        if self.cursor.y >= self.start_scroll_down && self.scroll_end < self.content.len() {
            if self.scroll_beg < self.scroll_end {
                self.scroll_beg += 1;
            }
            self.scroll_end += 1;
        } else {
            if self.get_current_line() < self.content.len() - 1 {
                self.cursor.move_y(1);
            }
        }
    }
    fn move_cur_up(&mut self) {
        if self.cursor.y == self.start_scroll_up && self.scroll_beg > 0
            || (self.cursor.y == 0 && self.scroll_beg != 0)
        {
            self.scroll_beg -= 1;
            if self.scroll_end > 0 {
                self.scroll_end -= 1;
            }
        } else {
            self.cursor.move_y(-1);
        }
    }

    //TODO: make this more like vim
    fn move_to_line(&mut self, line: usize) {
        if line >= self.content.len() - 1 {
            self.move_to_bottom();
            return;
        }

        self.scroll_beg = line - 1;
        self.scroll_end = self.scroll_beg + self.term_columns;
        self.cursor.y = 0; //self.term_columns as i32 / 2;
    }

    fn move_to_top(&mut self) {
        self.scroll_beg = 0;
        self.scroll_end = self.term_columns;
        self.cursor.y = 0;
    }

    fn move_to_bottom(&mut self) {
        self.scroll_end = self.content.len();
        self.scroll_beg = self.scroll_end - self.term_columns;
        self.cursor.y = self.term_columns - 1;
    }

    fn get_current_line(&self) -> usize {
        self.cursor.y + self.scroll_beg
    }

    fn align_cursor(&mut self) {
        //It's safe to unwrap here
        let cll = self
            .content
            .get(self.get_current_line())
            .unwrap()
            .len()
            .max(1)
            - 1;
        if cll < self.cursor.x {
            self.cursor.x = cll;
        }
    }

    //Return false to exit.
    fn handle_character(&mut self, buffer: Buffer) -> bool {
        if let Some(ch) = char_parser::parse_char(buffer) {
            match self.mode {
                InputMode::Normal => {
                    if let Some(command) = self.command_parser.parse_command(ch) {
                        match command {
                            NormalModeCommand::Quit => return Self::exit(),
                            NormalModeCommand::MoveDown => self.move_cur_down(),
                            NormalModeCommand::MoveUp => self.move_cur_up(),
                            NormalModeCommand::MoveLeft => self.cursor.move_x(-1),
                            NormalModeCommand::MoveRight => self.cursor.move_x(1),
                            NormalModeCommand::MoveToBottom => {
                                if let Some(nr_prefix) = self.command_parser.nr_prefix() {
                                    self.move_to_line(nr_prefix);
                                    self.command_parser.clear_nr_prefix();
                                } else {
                                    self.move_to_bottom();
                                }
                            }
                            NormalModeCommand::MoveToTop => {
                                if let Some(nr_prefix) = self.command_parser.nr_prefix() {
                                    self.move_to_line(nr_prefix);
                                    self.command_parser.clear_nr_prefix();
                                } else {
                                    self.move_to_top();
                                }
                            }
                            NormalModeCommand::EnterInsertMode => {
                                self.mode = InputMode::Insert;
                                self.changed = true;
                            }
                            NormalModeCommand::Append => {
                                self.mode = InputMode::Insert;
                                self.cursor.move_x(1);
                                self.changed = true;
                            }
                            NormalModeCommand::DeleteLine => {
                                for i in self.get_current_line()
                                    ..self.command_parser.nr_prefix().unwrap_or(1)
                                {
                                    self.content.remove(i);
                                }
                                self.command_parser.clear_nr_prefix();
                            }
                            _ => {}
                        }
                    }
                }
                InputMode::Insert => {
                    if let Some(command) = self.command_parser.parse_insert_mode_command(ch) {
                        match command {
                            InsertModeCommand::EnterNormalMode => {
                                self.mode = InputMode::Normal;
                                self.cursor.move_x(-1);
                                self.changed = true;
                            }
                            InsertModeCommand::Backspace => {
                                if self.cursor.x != 0 {
                                    let curr = self.get_current_line();
                                    if let Some(curr) = self.content.get_mut(curr) {
                                        if self.cursor.x - 1 < curr.len() {
                                            curr.content.remove(self.cursor.x - 1);
                                            self.cursor.move_x(-1);
                                        }
                                    }
                                }
                            }
                            InsertModeCommand::Delete => {
                                let curr = self.get_current_line();
                                if let Some(curr) = self.content.get_mut(curr) {
                                    if self.cursor.x < curr.len() {
                                        curr.content.remove(self.cursor.x);
                                    }
                                }
                            }
                            //Arrow Up
                            InsertModeCommand::MoveUp => {
                                self.move_cur_up();
                            }
                            //Arrow Down
                            InsertModeCommand::MoveDown => {
                                self.move_cur_down();
                            }
                            //Arrow Left
                            InsertModeCommand::MoveLeft => {
                                self.cursor.move_x(-1);
                            }
                            //Arrow Right
                            InsertModeCommand::MoveRight => {
                                self.cursor.move_x(1);
                            }
                            InsertModeCommand::Insert(c) => {
                                let curr = self.get_current_line();
                                if let Some(curr) = self.content.get_mut(curr) {
                                    curr.content.insert(self.cursor.x, c);
                                    self.cursor.move_x(1);
                                }
                            }
                        }
                    }
                }
            }
            true
        } else {
            true
        }
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        // let mut buf = Vec::new();
        // std::io::stdin().(&mut buf)?;
        // println!("{:?}", buf);
        let mut buffer = [0, 0, 0, 0];
        let mut stdin = io::stdin();

        // stdout.lock().flush().unwrap();

        stdin.read(&mut buffer)?;
        if !self.handle_character(buffer) {
            return Ok(false);
        }
        let x = char::from_u32(u32::from_le_bytes(buffer));
        // panic!("[DEBUG PANIC] Buffer: {:?}, char: {:?}", buffer, x);
        self.bottom_line = Some(Line::new(format!(
            "{} Line: {} {:?} Buffer: {:?}, char: {:?}",
            match self.mode {
                InputMode::Normal => "NORMAL",
                InputMode::Insert => "INSERT",
            },
            self.get_current_line(),
            self.cursor,
            buffer,
            x
        )));
        self.align_cursor();
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

        if let Some(bl) = &self.bottom_line {
            for c in &self.content[start..end - 1] {
                c.render();
            }
            bl.render();
        } else {
            for c in &self.content[start..end] {
                c.render();
            }
        }
        self.cursor.render(&self.mode);
        Self::flush()?;
        Ok(())
    }
}
