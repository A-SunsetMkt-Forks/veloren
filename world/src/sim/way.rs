use vek::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Way {
    /// Offset from chunk center in blocks (no more than half chunk width)
    pub offset: Vec2<i8>,
    /// Neighbor connections, one bit each
    pub neighbors: u8,
}

impl Way {
    pub fn is_way(&self) -> bool { self.neighbors != 0 }

    pub fn clear(&mut self) { self.neighbors = 0; }
}

#[derive(Copy, Clone, Debug)]
pub struct Path {
    pub width: f32, // Actually radius
}

impl Default for Path {
    fn default() -> Self { Self { width: 5.0 } }
}

impl Lerp for Path {
    type Output = Self;

    fn lerp_unclamped(from: Self, to: Self, factor: f32) -> Self::Output {
        Self {
            width: Lerp::lerp(from.width, to.width, factor),
        }
    }
}

impl Path {
    /// Return the number of blocks of headspace required at the given path
    /// distance
    /// TODO: make this generic over width
    pub fn head_space(&self, dist: f32) -> i32 { (8 - (dist * 0.25).powi(6).round() as i32).max(1) }

    /// Get the surface colour of a path given the surrounding surface color
    pub fn surface_color(&self, col: Rgb<u8>) -> Rgb<u8> { col.map(|e| (e as f32 * 0.7) as u8) }
}
