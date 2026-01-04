#[derive(Copy, Clone, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct IntVec2 {
    pub x: i32,
    pub y: i32,
}

impl IntVec2 {
    pub fn from(vec: &Vec2) -> Self {
	Self {
	    x: vec.x.round() as i32,
	    y: vec.y.round() as i32,
	}
    }
}
