#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Command {
    Invalid,
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
    Quit,
    MoveToBottom,
    MoveToTop,
    GoTo, //Intended for 'g'
}

#[derive(Debug)]
pub struct CommandParser {
    command_buffer: Vec<Command>,
    nr_prefix: Option<u128>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            command_buffer: Vec::new(),
            nr_prefix: None,
        }
    }
    fn handle_number_prefix(&mut self, c: char) {
        let nr = u128::from_str_radix(&c.to_string(), 10).unwrap(); //We are sure this is a number so
                                                                    //we can unwrap here

        if let Some(prev) = self.nr_prefix {
            self.nr_prefix = Some(prev * 10 + nr);
        } else {
            if nr != 0 {
                self.nr_prefix = Some(nr);
            }
        }
    }

    pub fn parse_command(&mut self, c: char) -> Option<Command> {
        match c {
            //Esc
            '\u{1B}' => {
                self.command_buffer.clear();
                None
            }
            //Ctrl-C
            '\u{3}' | 'q' => Some(Command::Quit),
            'j' => Some(Command::MoveDown),
            'k' => Some(Command::MoveUp),
            'h' => Some(Command::MoveLeft),
            'l' => Some(Command::MoveRight),
            'G' => Some(Command::MoveToBottom),
            'g' => {
                if self.command_buffer == vec![Command::GoTo] {
                    self.command_buffer.clear();
                    Some(Command::MoveToTop)
                } else {
                    // if let Some(Command::NumberPrefix(o)) = self.command_buffer.first().cloned() {
                    //     if self.command_buffer.get(1) == Some(&Command::GoTo) {
                    //         self.command_buffer.clear();
                    //         self.scroll_beg = o as usize;
                    //         self.scroll_end = self.scroll_beg + self.term_columns as usize;
                    //         self.cursor.y = 0;
                    //     }
                    // }
                    self.command_buffer.push(Command::GoTo);
                    None
                }
            }
            '0'..='9' => {
                self.handle_number_prefix(c);
                None
            }
            _ => None,
        }
    }

    pub fn nr_prefix(&self) -> Option<u128> {
        self.nr_prefix
    }
}

mod test {

    #[cfg(test)]
    use crate::command_parser::{Command, CommandParser};

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
    parse_command_test_wrapper!(parse_command_q, 'q', Some(Command::Quit));
    parse_command_test_wrapper!(parse_command_ctrl_c, '\u{3}', Some(Command::Quit));
    parse_command_test_wrapper!(parse_command_j, 'j', Some(Command::MoveDown));
    parse_command_test_wrapper!(parse_command_k, 'k', Some(Command::MoveUp));
    parse_command_test_wrapper!(parse_command_h, 'h', Some(Command::MoveLeft));
    parse_command_test_wrapper!(parse_command_l, 'l', Some(Command::MoveRight));
    parse_command_test_wrapper!(parse_command_capital_g, 'G', Some(Command::MoveToBottom));

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
        assert_eq!(cp.parse_command('g'), Some(Command::MoveToTop));
        assert!(cp.command_buffer.is_empty());
        assert_eq!(cp.nr_prefix, None);
    }
}
