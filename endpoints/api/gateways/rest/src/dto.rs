use serde::{Deserialize, Serialize};

/// شكل المستخدم في الـ REST — نسخة مستقلة عن موديل الدومين.
#[derive(Serialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub banned: bool,
}

impl From<kernel::user::User> for UserDto {
    fn from(u: kernel::user::User) -> Self {
        Self {
            id: u.id.to_string(),
            name: u.name,
            email: u.email,
            banned: u.banned,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUserBody {
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub salary_cents: i64,
}

#[derive(Serialize)]
pub struct OrderDto {
    pub id: String,
    pub total_cents: i64,
    pub status: &'static str,
    pub tenant: Option<String>,
}

impl From<kernel::order::Order> for OrderDto {
    fn from(o: kernel::order::Order) -> Self {
        Self {
            id: o.id.to_string(),
            total_cents: o.total.cents(),
            status: match o.status {
                kernel::order::OrderStatus::Pending => "pending",
                kernel::order::OrderStatus::Paid => "paid",
                kernel::order::OrderStatus::Cancelled => "cancelled",
            },
            tenant: o.tenant,
        }
    }
}

/// نتيجة فلو `account_overview` — مجمّعة من identity + orders + billing.
#[derive(Serialize)]
pub struct AccountOverviewDto {
    pub user: UserDto,
    pub orders: Vec<OrderDto>,
    pub total_spent_cents: i64,
    pub total_spent: String,
}

impl From<services::AccountOverview> for AccountOverviewDto {
    fn from(o: services::AccountOverview) -> Self {
        Self {
            user: o.user.into(),
            orders: o.orders.into_iter().map(OrderDto::from).collect(),
            total_spent_cents: o.total_spent.cents(),
            total_spent: o.total_spent.to_string(),
        }
    }
}
