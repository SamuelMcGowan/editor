use tinyvec::ArrayVec;

const ARRAY_SIZE: usize = 6;

struct Tree {
    root: Node,
}

#[derive(Debug, Clone, Copy)]
struct NodeInfo {
    total_bytes: usize,
    total_chars: usize,
    total_lines: usize,
}

enum Node {
    Internal(Box<NodeInternal>, NodeInfo),
    Leaf(Box<NodeLeaf>, NodeInfo),
}

impl Node {
    fn node_info(&self) -> NodeInfo {
        match self {
            Self::Internal(_, info) => *info,
            Self::Leaf(_, info) => *info,
        }
    }
}

struct NodeInternal(ArrayVec<[Node; ARRAY_SIZE]>);

struct NodeLeaf(ArrayVec<[Segment; ARRAY_SIZE]>);

impl NodeLeaf {
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

        for segment in &self.0 {
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
