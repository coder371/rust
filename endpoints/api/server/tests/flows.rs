//! تستات الجزء ❷: الفلوهات اللي بتلمس أكتر من خدمة في نفس الوقت.
//!
//! مكانها هنا لأن `api/server` هو المكان الوحيد اللي مسموح له
//! يشوف الستور والخدمات مع بعض (composition root).

use std::sync::Arc;

use kernel::{Actor, Money, OrderId, Role, UserId};
use services::{AuthService, ServiceHub, ServiceStatus};
use store::memory::{InMemoryOrderRepo, InMemoryUserRepo};

fn hub() -> ServiceHub {
    ServiceHub::new(
        Arc::new(InMemoryUserRepo::seeded()),
        Arc::new(InMemoryOrderRepo::seeded()),
        AuthService::new("admin".into(), "partner".into(), "customer".into()),
    )
}

fn admin() -> Actor {
    Actor {
        id: "admin".into(),
        role: Role::Admin,
        tenant: None,
    }
}

fn partner(tenant: &str) -> Actor {
    Actor {
        id: format!("partner:{tenant}"),
        role: Role::Partner,
        tenant: Some(tenant.to_owned()),
    }
}

#[tokio::test]
async fn registry_runs_every_service_together() {
    let hub = hub();

    assert_eq!(hub.registry().names(), ["identity", "orders", "billing"]);

    let report = hub.platform_health().await;
    assert_eq!(report.services.len(), 3);
    assert_eq!(report.status, ServiceStatus::Up);
    assert!(report.services.iter().all(|h| h.is_up()));
}

#[tokio::test]
async fn account_overview_joins_three_services() {
    // u1 عنده أوردرين مدفوعين: 250.00 + 150.00
    let overview = hub()
        .account_overview(&admin(), &UserId::new("u1"))
        .await
        .expect("admin يقدر يشوف الحساب");

    assert_eq!(overview.user.name, "Ahmed"); // identity
    assert_eq!(overview.orders.len(), 2); // orders
    assert_eq!(overview.total_spent, Money::from_cents(40_000)); // billing
}

#[tokio::test]
async fn account_overview_is_admin_only() {
    let err = hub()
        .account_overview(&Actor::anonymous(), &UserId::new("u1"))
        .await
        .expect_err("الزائر مالوش دعوة بالحساب");

    assert_eq!(err.code(), "FORBIDDEN");
}

#[tokio::test]
async fn admin_dashboard_aggregates_all_services() {
    let dashboard = hub().admin_dashboard(&admin()).await.expect("لوحة الإدارة");

    assert_eq!(dashboard.users_count, 3);
    assert_eq!(dashboard.revenue.paid_orders, 2);
    assert!(!dashboard.recent_orders.is_empty());
    assert_eq!(dashboard.services.len(), 3);
}

#[tokio::test]
async fn order_customer_stops_at_the_tenant_boundary() {
    let hub = hub();

    let customer = hub
        .order_customer(&partner("acme"), &OrderId::new("o1"))
        .await
        .expect("o1 تبع acme");
    assert_eq!(customer.name, "Ahmed");

    // o3 تبع globex — بيرجع NotFound مش Forbidden عشان مانسرّبش إنها موجودة
    let err = hub
        .order_customer(&partner("acme"), &OrderId::new("o3"))
        .await
        .expect_err("acme مايشوفش أوردر globex");
    assert_eq!(err.code(), "NOT_FOUND");
}
