use kernel::{
    Money, OrderId, UserId,
    order::{Order, OrderStatus},
    user::User,
};

pub(super) fn users() -> Vec<User> {
    vec![
        User {
            id: UserId::new("u1"),
            name: "Ahmed".into(),
            email: "ahmed@example.com".into(),
            salary: Money::from_cents(1_500_000),
            internal_notes: "قدّم على ترقية".into(),
            banned: false,
        },
        User {
            id: UserId::new("u2"),
            name: "Mona".into(),
            email: "mona@example.com".into(),
            salary: Money::from_cents(2_200_000),
            internal_notes: "تيم ليدر".into(),
            banned: false,
        },
        User {
            id: UserId::new("u3"),
            name: "Karim".into(),
            email: "karim@example.com".into(),
            salary: Money::from_cents(900_000),
            internal_notes: "تحت التجربة".into(),
            banned: true,
        },
    ]
}

pub(super) fn orders() -> Vec<Order> {
    vec![
        Order {
            id: OrderId::new("o1"),
            user_id: UserId::new("u1"),
            total: Money::from_cents(25_000),
            status: OrderStatus::Paid,
            tenant: Some("acme".into()),
        },
        Order {
            id: OrderId::new("o2"),
            user_id: UserId::new("u2"),
            total: Money::from_cents(40_000),
            status: OrderStatus::Pending,
            tenant: Some("acme".into()),
        },
        Order {
            id: OrderId::new("o3"),
            user_id: UserId::new("u1"),
            total: Money::from_cents(15_000),
            status: OrderStatus::Paid,
            tenant: Some("globex".into()),
        },
        Order {
            id: OrderId::new("o4"),
            user_id: UserId::new("u3"),
            total: Money::from_cents(70_000),
            status: OrderStatus::Cancelled,
            tenant: None,
        },
    ]
}
