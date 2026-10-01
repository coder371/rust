#[derive(Debug)]
pub struct AccessError;
impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AccessError")
    }
}

impl std::error::Error for AccessError {}