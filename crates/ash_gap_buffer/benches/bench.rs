use ash_gap_buffer::GapBuffer;
use divan::{bench, Bencher};

fn main() {
    divan::main();
}

#[bench(min_time = 0.25)]
fn push(bencher: Bencher) {
    let mut buf = GapBuffer::new();

    bencher.bench_local(|| {
        buf.push(0);
    })
}

#[bench]
fn move_gap(bencher: Bencher) {
    let mut buf = GapBuffer::new();
    buf.push_slice(b"hello, world, how are you???");

    bencher.bench_local(|| {
        buf.set_gap(buf.len());
        buf.set_gap(0);
    })
}
