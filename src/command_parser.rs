use crate::renderer::Buffer;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NormalModeCommand {
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
    Quit,
    MoveToBottom,
    MoveToTop,
    GoTo, //Intended for 'g'
    EnterInsertMode,
    Append,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InsertModeCommand {
    EnterNormalMode,
    Insert(char),
    Delete,
    Backspace,
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
}

#[derive(Debug)]
pub struct CommandParser {
    command_buffer: Vec<NormalModeCommand>,
    nr_prefix: Option<usize>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            command_buffer: Vec::new(),
            nr_prefix: None,
        }
    }
    fn handle_number_prefix(&mut self, c: char) {
        let nr = c.to_string().parse::<usize>().unwrap(); //We are sure this is a number so
                                                          //we can unwrap here

        if let Some(prev) = self.nr_prefix {
            self.nr_prefix = Some(prev * 10 + nr);
        } else if nr != 0 {
            //If the nr begins with 0 we ignore the 0
            self.nr_prefix = Some(nr);
        }
    }

    pub fn parse_insert_mode_command(&self, buffer: Buffer) -> Option<InsertModeCommand> {
        let zeros = buffer.iter().filter(|x| **x == 0).count();

        let values = buffer.len() - zeros;

        match values {
            1 => {
                match *buffer.first().unwrap() as char {
                    //Esc
                    '\u{1B}' => Some(InsertModeCommand::EnterNormalMode),
                    //Backspace
                    '\u{7F}' => Some(InsertModeCommand::Backspace),
                    c => {
                        if c.is_ascii_graphic() {
                            Some(InsertModeCommand::Insert(c))
                        } else {
                            None
                        }
                    }
                }
            }
            3 => {
                match buffer[..3] {
                    //Arrow Up
                    [27, 91, 65] => Some(InsertModeCommand::MoveUp),
                    //Arrow Down
                    [27, 91, 66] => Some(InsertModeCommand::MoveDown),
                    //Arrow Left
                    [27, 91, 68] => Some(InsertModeCommand::MoveLeft),
                    //Arrow Right
                    [27, 91, 67] => Some(InsertModeCommand::MoveRight),
                    _ => None,
                }
            }
            4 => {
                match buffer {
                    //Delete
                    [27, 91, 51, 126] => Some(InsertModeCommand::Delete),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn parse_command(&mut self, c: char) -> Option<NormalModeCommand> {
        match c {
            //Esc
            '\u{1B}' => {
                self.command_buffer.clear();
                None
            }
            //Ctrl-C
            '\u{3}' | 'q' => Some(NormalModeCommand::Quit),
            'j' => Some(NormalModeCommand::MoveDown),
            'k' => Some(NormalModeCommand::MoveUp),
            'h' => Some(NormalModeCommand::MoveLeft),
            'l' => Some(NormalModeCommand::MoveRight),
            'G' => Some(NormalModeCommand::MoveToBottom),
            'g' => {
                if self.command_buffer == vec![NormalModeCommand::GoTo] {
                    self.command_buffer.clear();
                    Some(NormalModeCommand::MoveToTop)
                } else {
                    // if let Some(NormalModeCommand::NumberPrefix(o)) = self.command_buffer.first().cloned() {
                    //     if self.command_buffer.get(1) == Some(&NormalModeCommand::GoTo) {
                    //         self.command_buffer.clear();
                    //         self.scroll_beg = o as usize;
                    //         self.scroll_end = self.scroll_beg + self.term_columns as usize;
                    //         self.cursor.y = 0;
                    //     }
                    // }
                    self.command_buffer.push(NormalModeCommand::GoTo);
                    None
                }
            }
            '0'..='9' => {
                self.handle_number_prefix(c);
                None
            }
            'i' => Some(NormalModeCommand::EnterInsertMode),
            'a' => Some(NormalModeCommand::Append),
            _ => None,
        }
    }

    pub fn nr_prefix(&self) -> Option<usize> {
        self.nr_prefix
    }

    pub fn clear_nr_prefix(&mut self) {
        self.nr_prefix = None;
    }
}

mod test {

    #[cfg(test)]
    use crate::command_parser::{CommandParser, NormalModeCommand};

    macro_rules! parse_command_test_wrapper {
        ($name: ident,$c: expr, $out: expr) => {
            #[test]
            fn $name() {
                let mut cp = CommandParser::new();
                assert_eq!(cp.parse_command($c), $out);
            }
        };
    }

    //We want to keep theese as separate tests TK
    parse_command_test_wrapper!(parse_command_q, 'q', Some(NormalModeCommand::Quit));
    parse_command_test_wrapper!(parse_command_ctrl_c, '\u{3}', Some(NormalModeCommand::Quit));
    parse_command_test_wrapper!(parse_command_j, 'j', Some(NormalModeCommand::MoveDown));
    parse_command_test_wrapper!(parse_command_k, 'k', Some(NormalModeCommand::MoveUp));
    parse_command_test_wrapper!(parse_command_h, 'h', Some(NormalModeCommand::MoveLeft));
    parse_command_test_wrapper!(parse_command_l, 'l', Some(NormalModeCommand::MoveRight));
    parse_command_test_wrapper!(
        parse_command_capital_g,
        'G',
        Some(NormalModeCommand::MoveToBottom)
    );

    #[test]
    fn parse_command_nr() {
        let mut cp = CommandParser::new();
        for i in '0'..='9' {
            assert_eq!(cp.parse_command(i), None);
        }
    }

    #[test]
    fn parse_command_gg() {
        let mut cp = CommandParser::new();
        assert_eq!(cp.parse_command('g'), None);
        assert_eq!(cp.parse_command('g'), Some(NormalModeCommand::MoveToTop));
        assert!(cp.command_buffer.is_empty());
        assert_eq!(cp.nr_prefix, None);
    }
}
