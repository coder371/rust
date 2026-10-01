use async_graphql::{Object, Result};

#[derive(Default)]
pub struct HelloQuery;

#[Object]
impl HelloQuery {
    /// إند بوينت فحص بسيط.
    async fn hello(&self) -> Result<&str> {
        Ok("Hello World!")
    }
}
