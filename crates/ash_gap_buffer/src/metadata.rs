use tinyvec::ArrayVec;

const ARRAY_SIZE: usize = 6;

struct Leaf {
    segments: ArrayVec<[Segment; ARRAY_SIZE]>,

    total_bytes: usize,
    total_chars: usize,
    total_lines: usize,
}

impl Leaf {
    fn byte_offset_to_segment(&self, offset: usize) -> Option<(&Segment, usize)> {
        self.find_after_offset(offset, |seg| seg.bytes)
    }

    fn char_offset_to_segment(&self, char_offset: usize) -> Option<(&Segment, usize)> {
        self.find_after_offset(char_offset, |seg| seg.chars)
    }

    fn find_after_offset(
        &self,
        offset: usize,
        get_size: impl Fn(&Segment) -> u8,
    ) -> Option<(&Segment, usize)> {
        let mut acc_offset = 0;

        for segment in &self.segments {
            let size = get_size(segment) as usize;
            acc_offset += size;

            if offset < acc_offset {
                let remainder = size - (acc_offset - offset);
                return Some((segment, remainder));
            }
        }

        None
    }
}

#[derive(Default)]
struct Segment {
    bytes: u8,
    chars: u8,
    newline_terminated: bool,
}
