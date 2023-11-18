use gap_buffer::GapBuffer;

pub mod gap_buffer;
pub mod term;
pub mod char_buffer;
mod style;

fn main() {
    println!("Hello, world!");
}

pub struct Editor {
    buffer: GapBuffer,
}

impl Editor {
    fn draw(&self) {}
}
