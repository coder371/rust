use async_graphql::InputObject;
use kernel::Page;

#[derive(InputObject, Default)]
pub struct PageInput {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

impl From<PageInput> for Page {
    fn from(value: PageInput) -> Self {
        let default = Page::default();
        Page::new(
            value.offset.unwrap_or(default.offset),
            value.limit.unwrap_or(default.limit),
        )
    }
}

/// هيلبر: `page_or_default(page)` بدل تكرار نفس الشرط في كل إند بوينت.
pub fn page_or_default(page: Option<PageInput>) -> Page {
    page.unwrap_or_default().into()
}
