use bitflags::bitflags;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Paste(String),
    Unknown,
}

impl Event {
    pub fn key_no_mods(key_code: KeyCode) -> Self {
        Self::Key(KeyEvent {
            key_code,
            modifiers: Modifiers::empty(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    pub fn new_with_mods(key_code: KeyCode, modifiers: Modifiers) -> Self {
        Self {
            key_code,
            modifiers,
        }
    }

    pub fn new(key_code: KeyCode) -> Self {
        Self {
            key_code,
            modifiers: Modifiers::empty(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Fn(u8),

    Tab,
    Newline,
    Return,

    Escape,

    Up,
    Down,
    Right,
    Left,

    End,
    Home,

    Insert,
    Delete,
    Backspace,

    PageUp,
    PageDown,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Modifiers: u8 {
        const EMPTY = 0;

        const SHIFT = 0b0001;
        const ALT   = 0b0010;
        const CTRL  = 0b0100;
        const META  = 0b1000;
    }
}
