use gap_buffer::GapBuffer;

pub mod gap_buffer;
pub mod term;

fn main() {
    println!("Hello, world!");
}

pub struct Editor {
    buffer: GapBuffer,
}

impl Editor {
    fn draw(&self) {}
}
