use super::*;

pub type ZLevelID = &'static str;
pub type SpriteID = u64;

pub struct Position {
    pub x: f32,
    pub y: f32,
    z_level: ZLevelID
}

pub struct Visible {
    pub sprite: SpriteID
}

pub struct PositionTiled {
    pub x: u32,
    pub y: u32,
    pub z_level: ZLevelID
}

pub struct Camera {
    pub view_range: u16,
    pub offset: (u32, u32)
}

define_component!(Position);
define_component!(Visible);
define_component!(PositionTiled);
define_component!(Camera);