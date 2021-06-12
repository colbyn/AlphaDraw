use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub logical_size: Vector2I,
    pub backing_scale_factor: f32,
}

impl WindowSize {
    #[inline]
    pub fn device_size(&self) -> Vector2I {
        (self.logical_size.to_f32() * self.backing_scale_factor).to_i32()
    }
}


