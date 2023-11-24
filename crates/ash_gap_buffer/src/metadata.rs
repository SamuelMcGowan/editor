struct Node {
    kind: NodeKind,
    info: NodeInfo,
}

enum NodeKind {
    Internal { children: Vec<Node> },
    Leaf { segments: Vec<Segment> },
}

struct NodeInfo {
    bytes: usize,
    chars: usize,
    lines: usize,
}

struct Segment {
    bytes: u8,
    chars: u8,
    has_line_break: bool,
}
