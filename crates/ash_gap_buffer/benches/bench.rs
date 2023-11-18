use ash_gap_buffer::GapBuffer;
use divan::{bench, Bencher};

fn main() {
    divan::main();
}

#[bench(min_time = 0.5)]
fn push(bencher: Bencher) {
    let mut buf = GapBuffer::new();

    bencher.bench_local(|| {
        buf.push(0);
    })
}
