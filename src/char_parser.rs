use crate::renderer::Buffer;

pub enum Character {
    Display(char),
    Esc,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

pub fn parse_char(buffer: Buffer) -> Option<Character> {
    let zeros = buffer.iter().filter(|x| **x == 0).count();

    let values = buffer.len() - zeros;

    match values {
        1 => {
            match *buffer.first().unwrap() as char {
                //Esc
                '\u{1B}' => Some(Character::Esc),
                //Backspace
                '\u{7F}' => Some(Character::Backspace),
                ch => {
                    if ch.is_ascii_graphic() || ch == ' ' {
                        Some(Character::Display(ch))
                    } else {
                        None
                    }
                }
            }
        }
        3 => {
            match buffer[..3] {
                //Arrow Up
                [27, 91, 65] => Some(Character::ArrowUp),
                //Arrow Down
                [27, 91, 66] => Some(Character::ArrowDown),
                //Arrow Left
                [27, 91, 68] => Some(Character::ArrowLeft),
                //Arrow Right
                [27, 91, 67] => Some(Character::ArrowRight),
                _ => None,
            }
        }
        4 => {
            match buffer {
                //Delete
                [27, 91, 51, 126] => Some(Character::Delete),
                _ => None,
            }
        }
        _ => None,
    }
}
