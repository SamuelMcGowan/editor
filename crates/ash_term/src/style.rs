#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,

    #[default]
    Default = 9,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    #[default]
    Normal,
    Bold,
    Dim,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    #[default]
    Block,
    Underscore,
    Bar,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,

    pub weight: Weight,
    pub underline: bool,
}

impl Style {
    pub const EMPTY: Self = Style {
        fg: Color::Default,
        bg: Color::Default,

        weight: Weight::Normal,
        underline: false,
    };
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorStyle {
    pub shape: CursorShape,
    pub blinking: bool,
}

impl CursorStyle {
    pub const EMPTY: Self = CursorStyle {
        shape: CursorShape::Bar,
        blinking: false,
    };
}
