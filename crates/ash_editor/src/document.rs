use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufWriter, Write},
    ops::ControlFlow,
    path::PathBuf,
};
use unicode_width::UnicodeWidthStr;

use anyhow::{Context, Result};
use ash_term::units::OffsetUsize;
use crop::{Rope, RopeSlice};

#[derive(Default)]
pub struct Document {
    rope: Rope,
    path: Option<PathBuf>,

    /// Cursor position, as a byte index.
    cursor_index: usize,

    /// Column to try to move to when moving (in cells).
    target_column: Option<usize>,

    /// Scroll offset, in cells.
    scroll_offset: OffsetUsize,
}

impl Document {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let rope = if let Some(path) = &path {
            // TODO: do this properly
            let source = fs::read_to_string(path).context("couldn't read file")?;
            Rope::from(source)
        } else {
            Rope::new()
        };

        let cursor_index = rope.byte_len();

        Ok(Self {
            rope,
            path,
            cursor_index,
            ..Default::default()
        })
    }

    pub fn save_file(&self) {
        let snapshot = self.rope.clone();

        if let Some(path) = self.path.clone() {
            // TODO: report errors properly
            std::thread::spawn(move || {
                let mut file = BufWriter::new(File::create(path).expect("failed to open file"));
                for chunk in snapshot.chunks() {
                    file.write_all(chunk.as_bytes())
                        .expect("failed to write to file");
                }
            });
        }
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn scroll_offset(&self) -> OffsetUsize {
        self.scroll_offset
    }

    /// The cursor offset, in cells.
    pub fn cursor_offset(&self) -> OffsetUsize {
        let line = self.rope.line_of_byte(self.cursor_index);
        let line_start = self.rope.byte_of_line(line);

        // Fine to sum up the widths of each chunk - the `width` implementation just
        // sums the character widths, so it seems there's nothing contextual
        // that is lost by splitting up a string.
        let column: usize = self
            .rope
            .byte_slice(line_start..self.cursor_index)
            .chunks()
            .map(|s| s.width())
            .sum();

        OffsetUsize::new(column, line)
    }

    pub fn scroll_to_show_cursor(&mut self, size: OffsetUsize) {
        let cursor_offset = self.cursor_offset();

        if cursor_offset.x < self.scroll_offset.x {
            self.scroll_offset.x = cursor_offset.x;
        } else if cursor_offset.x >= self.scroll_offset.x + size.x {
            self.scroll_offset.x = cursor_offset.x - size.x + 1;
        }

        if cursor_offset.y < self.scroll_offset.y {
            self.scroll_offset.y = cursor_offset.y;
        } else if cursor_offset.y >= self.scroll_offset.y + size.y {
            self.scroll_offset.y = cursor_offset.y - size.y + 1;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        self.rope.insert(self.cursor_index, s);
        self.cursor_index += s.len();
        self.target_column = None;
    }

    pub fn insert_str_after(&mut self, s: &str) {
        self.rope.insert(self.cursor_index, s);
        self.target_column = None;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.insert_str(ch.encode_utf8(&mut [0; 4]));
    }

    pub fn insert_char_after(&mut self, ch: char) {
        self.insert_str_after(ch.encode_utf8(&mut [0; 4]));
    }

    pub fn backspace(&mut self) {
        if let Some(prev) = self.grapheme_before_cursor() {
            let prev_len = prev.len();
            self.rope
                .delete((self.cursor_index - prev_len)..self.cursor_index);
            self.cursor_index -= prev_len;
        }
        self.target_column = None;
    }

    pub fn delete(&mut self) {
        if let Some(next) = self.grapheme_after_cursor() {
            self.rope
                .delete(self.cursor_index..(self.cursor_index + next.len()));
        }
        self.target_column = None;
    }

    pub fn move_left(&mut self) {
        if let Some(prev) = self.grapheme_before_cursor() {
            self.cursor_index -= prev.len();
        }
        self.target_column = None;
    }

    pub fn move_right(&mut self) {
        if let Some(next) = self.grapheme_after_cursor() {
            self.cursor_index += next.len();
        }
        self.target_column = None;
    }

    pub fn move_up(&mut self) {
        self.move_vertical(-1);
    }

    pub fn move_down(&mut self) {
        self.move_vertical(1);
    }

    pub fn move_home(&mut self) {
        self.go_to_offset(OffsetUsize::new(0, self.cursor_offset().y));
        self.target_column = None;
    }

    pub fn move_end(&mut self) {
        let (_, line) = self.current_line();
        let line_width = line.chunks().map(|chunk| chunk.width()).sum();
        self.go_to_offset(OffsetUsize::new(line_width, self.cursor_offset().y));
        self.target_column = None;
    }

    pub fn move_vertical(&mut self, n: isize) {
        let prev_cursor_index = self.cursor_index;

        'main: {
            let cursor_offset = self.cursor_offset();

            let Some(new_offset_y) = cursor_offset.y.checked_add_signed(n) else {
                self.cursor_index = 0;
                self.target_column = Some(0);
                break 'main;
            };

            if new_offset_y >= self.rope.line_len() {
                self.cursor_index = self.rope.byte_len();

                let num_lines = self.rope.line_len();
                self.target_column = Some(match num_lines {
                    0 => 0,
                    _ => self
                        .rope
                        .line(num_lines - 1)
                        .chunks()
                        .map(|chunk| chunk.width())
                        .sum(),
                });

                break 'main;
            }

            let new_offset_x = match self.target_column {
                Some(col) => col,
                None => {
                    let col = cursor_offset.x;
                    self.target_column = Some(col);
                    col
                }
            };

            self.go_to_offset(OffsetUsize::new(new_offset_x, new_offset_y));
        }

        if self.cursor_index == prev_cursor_index {
            self.target_column = None;
        }
    }

    fn go_to_offset(&mut self, offset: OffsetUsize) {
        if offset.y >= self.rope.line_len() {
            self.cursor_index = self.rope.byte_len();
            return;
        };

        let line = self.rope.line(offset.y);
        let line_start = self.rope.byte_of_line(offset.y);

        let byte_offset = line.graphemes().try_fold((0, 0), |(acc, off), grapheme| {
            let end = acc + grapheme.width();
            if offset.x >= end {
                ControlFlow::Continue((end, off + grapheme.len()))
            } else {
                ControlFlow::Break(off)
            }
        });

        let byte_offset = match byte_offset {
            ControlFlow::Break(off) => off,
            ControlFlow::Continue((_, off)) => off,
        };

        self.cursor_index = line_start + byte_offset;
    }

    fn grapheme_before_cursor(&self) -> Option<Cow<str>> {
        self.rope_before_cursor().graphemes().next_back()
    }

    fn grapheme_after_cursor(&self) -> Option<Cow<str>> {
        self.rope_after_cursor().graphemes().next()
    }

    fn rope_before_cursor(&self) -> RopeSlice {
        self.rope.byte_slice(..self.cursor_index)
    }

    fn rope_after_cursor(&self) -> RopeSlice {
        self.rope.byte_slice(self.cursor_index..)
    }

    fn current_line(&self) -> (usize, RopeSlice) {
        let line_num = self.rope.line_of_byte(self.cursor_index);

        let slice = if line_num == self.rope.line_len() {
            self.rope.byte_slice(self.cursor_index..)
        } else {
            self.rope.line(line_num)
        };

        (line_num, slice)
    }
}

pub trait RopeExt {
    fn has_trailing_newline(&self) -> bool;
}

impl RopeExt for Rope {
    fn has_trailing_newline(&self) -> bool {
        match self.chunks().last() {
            Some(chunk) => chunk.ends_with('\n'),
            None => true,
        }
    }
}
