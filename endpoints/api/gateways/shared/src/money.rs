use async_graphql::SimpleObject;
use kernel::Money;

/// عرض الفلوس في السكيما: السنت للحسابات، والنص للعرض.
#[derive(SimpleObject)]
#[graphql(name = "Money")]
pub struct MoneyView {
    pub cents: i64,
    pub formatted: String,
}

impl From<Money> for MoneyView {
    fn from(value: Money) -> Self {
        Self {
            cents: value.cents(),
            formatted: value.to_string(),
        }
    }
}
