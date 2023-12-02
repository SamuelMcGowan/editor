use std::collections::HashMap;

use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use maplit::hashmap;

use crate::editor::Mode;

#[derive(Debug, Clone)]
pub enum Action {
    Combo(Vec<Action>),

    InsertChar(char),
    InsertCharAfter(char),

    InsertString(String),
    InsertStringAfter(String),

    Backspace,
    Delete,

    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    MoveHome,
    MoveEnd,

    SetMode(Mode),

    Quit,
}

pub struct KeyMap {
    pub all: HashMap<KeyEvent, Action>,
    pub normal: HashMap<KeyEvent, Action>,
    pub insert: HashMap<KeyEvent, Action>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::basic()
    }
}

impl KeyMap {
    pub fn basic() -> Self {
        let all = hashmap! {
            KeyEvent::new(KeyCode::Left) => Action::MoveLeft,
            KeyEvent::new(KeyCode::Right) => Action::MoveRight,
            KeyEvent::new(KeyCode::Up) => Action::MoveUp,
            KeyEvent::new(KeyCode::Down) => Action::MoveDown,

            KeyEvent::new(KeyCode::Home) => Action::MoveHome,
            KeyEvent::new(KeyCode::End) => Action::MoveEnd,
        };

        let normal = hashmap! {
            KeyEvent::new(KeyCode::Char('i')) => Action::SetMode(Mode::Insert),

            KeyEvent::new(KeyCode::Char('o')) => Action::Combo(vec![
                Action::MoveEnd,
                Action::InsertChar('\n'),
                Action::SetMode(Mode::Insert),
            ]),
            KeyEvent::new(KeyCode::Char('O')) => Action::Combo(vec![
                Action::MoveHome,
                Action::InsertCharAfter('\n'),
                Action::SetMode(Mode::Insert),
            ]),

            KeyEvent::new(KeyCode::Char('d')) => Action::Delete,

            KeyEvent::new(KeyCode::Char('h')) => Action::MoveLeft,
            KeyEvent::new(KeyCode::Char('l')) => Action::MoveRight,
            KeyEvent::new(KeyCode::Char('k')) => Action::MoveUp,
            KeyEvent::new(KeyCode::Char('j')) => Action::MoveDown,

            KeyEvent::new(KeyCode::Char('q')) => Action::Quit,
        };

        let insert = hashmap! {
            KeyEvent::new(KeyCode::Backspace) => Action::Backspace,
            KeyEvent::new(KeyCode::Delete) => Action::Delete,
            KeyEvent::new(KeyCode::Escape) => Action::SetMode(Mode::Normal),
        };

        Self {
            all,
            normal,
            insert,
        }
    }

    pub fn get_action(&self, mode: Mode, event: Event) -> Option<Action> {
        match mode {
            Mode::Normal => match event {
                Event::Paste(s) => Some(Action::InsertString(s)),
                Event::Key(key) => self
                    .normal
                    .get(&key)
                    .cloned()
                    .or_else(|| self.all.get(&key).cloned()),
                _ => None,
            },

            Mode::Insert => match event {
                Event::Paste(s) => Some(Action::InsertString(s)),

                Event::Key(KeyEvent {
                    key_code: KeyCode::Char(ch),
                    modifiers: Modifiers::EMPTY,
                }) => Some(Action::InsertChar(ch)),

                Event::Key(KeyEvent {
                    key_code: KeyCode::Return,
                    modifiers: Modifiers::EMPTY,
                }) => Some(Action::InsertChar('\n')),

                Event::Key(key) => self
                    .insert
                    .get(&key)
                    .cloned()
                    .or_else(|| self.all.get(&key).cloned()),

                _ => None,
            },
        }
    }
}
