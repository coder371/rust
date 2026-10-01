#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    pub offset: u32,
    pub limit: u32,
}

impl Page {
    pub const MAX_LIMIT: u32 = 100;

    pub fn new(offset: u32, limit: u32) -> Self {
        Self {
            offset,
            limit: limit.clamp(1, Self::MAX_LIMIT),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}
