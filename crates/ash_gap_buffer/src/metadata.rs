pub enum Node {
    Internal(NodeInternal),
    Leaf(NodeLeaf),
}

impl Node {
    pub fn text_info(&self) -> TextInfo {
        match self {
            Node::Internal(internal) => internal.text_info,
            Node::Leaf(leaf) => leaf.text_info(),
        }
    }

    fn before_byte(&self, byte_offset: usize) -> Option<TextInfo> {
        match self {
            Node::Internal(internal) => {
                if let Some(text_info) = internal.left.before_byte(byte_offset) {
                    return Some(text_info);
                }

                let left_info = internal.left.text_info();

                if let Some(text_info) = internal.right.before_byte(left_info.bytes + byte_offset) {
                    return Some(text_info);
                }

                None
            }

            Node::Leaf(leaf) => {
                if byte_offset < leaf.bytes as usize {
                    Some(leaf.text_info())
                } else {
                    None
                }
            }
        }
    }
}

pub struct NodeInternal {
    left: Box<Node>,
    right: Box<Node>,
    text_info: TextInfo,
}

pub struct NodeLeaf {
    bytes: u8,
    chars: u8,
    has_line_break: bool,
}

impl NodeLeaf {
    fn text_info(&self) -> TextInfo {
        TextInfo {
            bytes: self.bytes as usize,
            chars: self.chars as usize,
            lines: if self.has_line_break { 1 } else { 0 },
        }
    }

    fn byte_to_char(&self, byte_offset: u8) -> Option<u8> {
        if byte_offset > self.bytes {
            None
        } else {
            let char_width = self.bytes / self.chars;
            Some(byte_offset * char_width)
        }
    }

    fn char_width(&self) -> u8 {
        self.bytes / self.chars
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextInfo {
    bytes: usize,
    chars: usize,
    lines: usize,
}
