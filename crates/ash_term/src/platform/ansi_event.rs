use std::io;
use std::time::Instant;

use super::input::PollingStdin;
use super::Events;
use crate::event::*;

#[derive(Default)]
pub struct AnsiEvents {
    stdin: PollingStdin,
}

impl Events for AnsiEvents {
    fn read_with_deadline(&mut self, deadline: Instant) -> io::Result<Option<Event>> {
        let Some(bytes) = self.stdin.read_with_deadline(deadline)? else {
            return Ok(None);
        };
        let event = parse_event(bytes.as_slice()).unwrap_or(Event::Unknown);
        Ok(Some(event))
    }
}

pub fn parse_event(bytes: &[u8]) -> Option<Event> {
    let (&first, rest) = bytes.split_first()?;

    let event = match first {
        b'\x1b' => {
            match rest {
                b"" | b"\x1b" => Event::key_no_mods(KeyCode::Escape),

                b"[" => Event::Key(KeyEvent {
                    key_code: KeyCode::Char('['),
                    modifiers: Modifiers::ALT,
                }),

                // VT sequence.
                [b'[', rest @ .., b'~'] => {
                    let (key_code, modifiers) =
                        if let Some(idx) = rest.iter().position(|&byte| byte == b';') {
                            let (key_code, modifiers) = rest.split_at(idx);
                            Some((key_code, parse_modifiers(&modifiers[1..])?))
                        } else {
                            Some((rest, Modifiers::empty()))
                        }?;

                    let key_code = match key_code {
                        b"1" => KeyCode::Home,
                        b"2" => KeyCode::Insert,
                        b"3" => KeyCode::Delete,
                        b"4" => KeyCode::End,
                        b"5" => KeyCode::PageUp,
                        b"6" => KeyCode::PageDown,
                        b"7" => KeyCode::Home,
                        b"8" => KeyCode::End,

                        b"11" => KeyCode::Fn(1),
                        b"12" => KeyCode::Fn(2),
                        b"13" => KeyCode::Fn(3),
                        b"14" => KeyCode::Fn(4),
                        b"15" => KeyCode::Fn(5),

                        // No this isn't a typo, `16` is skipped.
                        b"17" => KeyCode::Fn(6),
                        b"18" => KeyCode::Fn(7),
                        b"19" => KeyCode::Fn(8),
                        b"20" => KeyCode::Fn(9),
                        b"21" => KeyCode::Fn(10),

                        // Who needs more than 10 function keys? Let's leave it at that.
                        _ => return None,
                    };

                    Event::Key(KeyEvent {
                        key_code,
                        modifiers,
                    })
                }

                // XTerm sequence.
                [b'[', modifiers @ .., key_code] => {
                    let key_code = match key_code {
                        b'A' => KeyCode::Up,
                        b'B' => KeyCode::Down,
                        b'C' => KeyCode::Right,
                        b'D' => KeyCode::Left,

                        b'F' => KeyCode::End,
                        b'H' => KeyCode::Home,

                        b'P' => KeyCode::Fn(1),
                        b'Q' => KeyCode::Fn(2),
                        b'R' => KeyCode::Fn(3),
                        b'S' => KeyCode::Fn(4),

                        _ => return None,
                    };

                    let modifiers =
                        if let Some(index) = modifiers.iter().position(|&byte| byte == b';') {
                            modifiers.split_at(index.saturating_add(1)).1
                        } else {
                            modifiers
                        };

                    let modifiers = if modifiers.is_empty() {
                        Modifiers::empty()
                    } else {
                        parse_modifiers(modifiers)?
                    };

                    Event::Key(KeyEvent {
                        key_code,
                        modifiers,
                    })
                }

                [c] => {
                    let mut key_event = decode_byte(*c);
                    key_event.modifiers |= Modifiers::ALT;
                    Event::Key(key_event)
                }

                _ => return None,
            }
        }

        _ => decode_bytes(bytes)?,
    };

    Some(event)
}

fn parse_modifiers(bytes: &[u8]) -> Option<Modifiers> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .map(|byte| Modifiers::from_bits_truncate(byte.saturating_sub(1)))
}

/// Decode a character, handling the control keys, control characters* and
/// utf-8.
///
/// *Not to be confused with control keys.
fn decode_bytes(bytes: &[u8]) -> Option<Event> {
    match bytes {
        [] => None,
        [b] => Some(Event::Key(decode_byte(*b))),
        _ => {
            let s = std::str::from_utf8(bytes).ok()?;
            let kind = if s.chars().nth(1).is_some() {
                Event::Paste(s.to_owned())
            } else {
                Event::Key(KeyEvent::new(KeyCode::Char(s.chars().next()?)))
            };
            Some(kind)
        }
    }
}

fn decode_byte(byte: u8) -> KeyEvent {
    match byte {
        b'\t' => KeyEvent::new(KeyCode::Tab),
        b'\n' => KeyEvent::new(KeyCode::Newline),
        b'\r' => KeyEvent::new(KeyCode::Return),

        b'\x7f' => KeyEvent::new(KeyCode::Backspace),
        b'\x17' => KeyEvent {
            key_code: KeyCode::Backspace,
            modifiers: Modifiers::CTRL,
        },

        b if b < 27 => KeyEvent {
            key_code: KeyCode::Char((b'A' + b - 1) as char),
            modifiers: Modifiers::CTRL,
        },

        _ => KeyEvent::new(KeyCode::Char(byte as char)),
    }
}
