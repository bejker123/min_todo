use crate::char_parser::Character;

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
    Delete,
    DeleteLine,
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

    pub fn parse_insert_mode_command(&self, c: Character) -> Option<InsertModeCommand> {
        match c {
            //Esc
            Character::Esc => Some(InsertModeCommand::EnterNormalMode),
            //Backspace
            Character::Backspace => Some(InsertModeCommand::Backspace),
            Character::Display(c) => {
                if c.is_ascii_graphic() || c == ' ' {
                    Some(InsertModeCommand::Insert(c))
                } else {
                    None
                }
            }
            //Arrow Up
            Character::ArrowUp => Some(InsertModeCommand::MoveUp),
            //Arrow Down
            Character::ArrowDown => Some(InsertModeCommand::MoveDown),
            //Arrow Left
            Character::ArrowLeft => Some(InsertModeCommand::MoveLeft),
            //Arrow Right
            Character::ArrowRight => Some(InsertModeCommand::MoveRight),
            //Delete
            Character::Delete => Some(InsertModeCommand::Delete),
            _ => None,
        }
    }

    pub fn parse_command(&mut self, c: Character) -> Option<NormalModeCommand> {
        match c {
            //Esc
            Character::Esc => {
                self.command_buffer.clear();
                None
            }
            //Ctrl-C
            Character::Display('\u{3}') | Character::Display('q') => Some(NormalModeCommand::Quit),
            Character::ArrowDown | Character::Display('j') => Some(NormalModeCommand::MoveDown),
            Character::ArrowUp | Character::Display('k') => Some(NormalModeCommand::MoveUp),
            Character::ArrowLeft | Character::Display('h') => Some(NormalModeCommand::MoveLeft),
            Character::ArrowRight | Character::Display('l') => Some(NormalModeCommand::MoveRight),
            Character::Display('G') => Some(NormalModeCommand::MoveToBottom),
            Character::Display('g') => {
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
            Character::Display('0'..='9') => {
                if let Character::Display(c) = c {
                    self.handle_number_prefix(c);
                }
                None
            }
            Character::Display('i') => Some(NormalModeCommand::EnterInsertMode),
            Character::Display('a') => Some(NormalModeCommand::Append),
            Character::Display('d') => {
                if self.command_buffer == vec![NormalModeCommand::Delete] {
                    self.command_buffer.clear();
                    Some(NormalModeCommand::DeleteLine)
                } else {
                    self.command_buffer.push(NormalModeCommand::Delete);
                    None
                }
            }
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

#[cfg(test)]
mod test {

    use crate::char_parser::Character;
    use crate::command_parser::{CommandParser, InsertModeCommand, NormalModeCommand};

    macro_rules! parse_insert_command_test_wrapper {
        ($name: ident,$c: expr, $out: expr) => {
            #[test]
            fn $name() {
                let cp = CommandParser::new();
                assert_eq!(cp.parse_insert_mode_command($c), $out);
            }
        };
    }

    parse_insert_command_test_wrapper!(
        parse_insert_command_esc,
        Character::Esc,
        Some(InsertModeCommand::EnterNormalMode)
    );

    parse_insert_command_test_wrapper!(
        parse_insert_command_delete,
        Character::Delete,
        Some(InsertModeCommand::Delete)
    );

    parse_insert_command_test_wrapper!(
        parse_insert_command_arr_down,
        Character::ArrowDown,
        Some(InsertModeCommand::MoveDown)
    );

    parse_insert_command_test_wrapper!(
        parse_insert_command_arr_up,
        Character::ArrowUp,
        Some(InsertModeCommand::MoveUp)
    );

    parse_insert_command_test_wrapper!(
        parse_insert_command_arr_left,
        Character::ArrowLeft,
        Some(InsertModeCommand::MoveLeft)
    );

    parse_insert_command_test_wrapper!(
        parse_insert_command_arr_right,
        Character::ArrowRight,
        Some(InsertModeCommand::MoveRight)
    );

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
    parse_command_test_wrapper!(
        parse_command_q,
        Character::Display('q'),
        Some(NormalModeCommand::Quit)
    );
    parse_command_test_wrapper!(
        parse_command_ctrl_c,
        Character::Display('\u{3}'),
        Some(NormalModeCommand::Quit)
    );
    parse_command_test_wrapper!(
        parse_command_j,
        Character::Display('j'),
        Some(NormalModeCommand::MoveDown)
    );
    parse_command_test_wrapper!(
        parse_command_k,
        Character::Display('k'),
        Some(NormalModeCommand::MoveUp)
    );
    parse_command_test_wrapper!(
        parse_command_h,
        Character::Display('h'),
        Some(NormalModeCommand::MoveLeft)
    );
    parse_command_test_wrapper!(
        parse_command_l,
        Character::Display('l'),
        Some(NormalModeCommand::MoveRight)
    );
    parse_command_test_wrapper!(
        parse_command_capital_g,
        Character::Display('G'),
        Some(NormalModeCommand::MoveToBottom)
    );

    #[test]
    fn parse_command_nr() {
        let mut cp = CommandParser::new();
        for i in '0'..='9' {
            assert_eq!(cp.parse_command(Character::Display(i)), None);
        }
    }

    #[test]
    fn parse_command_gg() {
        let mut cp = CommandParser::new();
        assert_eq!(cp.parse_command(Character::Display('g')), None);
        assert_eq!(
            cp.parse_command(Character::Display('g')),
            Some(NormalModeCommand::MoveToTop)
        );
        assert!(cp.command_buffer.is_empty());
        assert_eq!(cp.nr_prefix, None);
    }
}
