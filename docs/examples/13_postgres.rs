//! الدرس 13 — PostgreSQL مع sqlx
//! قبل ما تشغّل:  docker compose up -d postgres
//! شغّل:          cargo run --example 13_postgres
//!
//! ═══ الفلسفة ═══
//! في Node عندك طيف: pg (SQL خام) ← knex (query builder) ← Prisma/TypeORM (ORM).
//! في Rust نفس الطيف:  sqlx (SQL خام + type safety) ← sea-query ← SeaORM / Diesel.
//! sqlx هو الأشهر، وفلسفته: "اكتب SQL حقيقي، واحنا نضمنلك الأنواع".
//!
//!  - query_as::<_, T>(sql)   → runtime check (اللي هنستخدمه هنا — مش محتاج DB وقت الـ compile)
//!  - query_as!(T, sql)       → compile-time check! الـ macro بيتصل بالـ DB وقت الـ build
//!    ويتأكد إن الـ SQL صح والـ columns بنفس الأنواع. لو غلطت في اسم column
//!    → الكود مش هيعمل compile. (ولـ CI فيه `cargo sqlx prepare` offline mode)
//!
//!  - Pool: زي pg.Pool بالظبط. اعمله مرة واحدة واعمله clone (رخيص — Arc من جوه) لكل حتة.
//!  - الـ parameters: $1, $2 + .bind()  → مفيش SQL injection.
//!    ومن sqlx 0.9: query() بتقبل &'static str بس (SqlSafeStr) — لو حاولت تبعتلها String عملتها
//!    بـ format! مش هتعمل compile. الـ API نفسه بيمنعك من الغلط. للـ SQL الديناميك: QueryBuilder (تحت).

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{FromRow, Postgres, Row, Transaction};
use uuid::Uuid;

// FromRow = الـ mapping من row لـ struct (أسماء الـ fields = أسماء الـ columns)
#[derive(Debug, FromRow)]
struct User {
    id: Uuid,
    email: String,
    name: String,
    balance: i64,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct OrderRow {
    id: Uuid,
    total: i64,
    status: String,
    meta: sqlx::types::Json<serde_json::Value>, // JSONB ↔ أي نوع بيعمل Serialize/Deserialize
}

// Repository pattern: الـ struct ده بيمسك الـ pool ويعرض functions typed
// (ده بديل الـ "Model" في mongoose/sequelize)
#[derive(Clone)]
struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    async fn create(&self, email: &str, name: &str, balance: i64) -> Result<User, sqlx::Error> {
        // RETURNING * → نرجع الـ row بعد الـ insert في نفس الـ round-trip
        sqlx::query_as::<_, User>(
            "INSERT INTO users (id, email, name, balance) VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(name)
        .bind(balance)
        .fetch_one(&self.pool)
        .await
    }

    // fetch_optional → Option: مش موجود ≠ error
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
    }

    async fn list(&self, limit: i64) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM users ORDER BY created_at DESC LIMIT $1")
            .bind(limit)
            .fetch_all(&self.pool)
            .await
    }
}

// Errors الـ domain — بنحوّل errors الـ DB لمعنى في الـ business
#[derive(Debug, thiserror::Error)]
enum CheckoutError {
    #[error("user not found")]
    UserNotFound,
    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: i64, need: i64 },
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// Transaction: خصم الرصيد + إنشاء order — يا الاتنين يحصلوا يا ولا حاجة.
/// في Node (pg): BEGIN / try { ... COMMIT } catch { ROLLBACK } finally { client.release() }
/// في Rust: لو الـ tx اتعملها drop من غير commit (بسبب `?` مثلاً) → ROLLBACK أوتوماتيك (RAII تاني!)
async fn checkout(pool: &PgPool, user_id: Uuid, amount: i64) -> Result<Uuid, CheckoutError> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // FOR UPDATE → lock على الـ row لحد آخر الـ transaction (يمنع race بين طلبين في نفس الوقت)
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM users WHERE id = $1 FOR UPDATE")
        .bind(user_id)
        .fetch_optional(&mut *tx) // &mut *tx = استخدم الـ transaction كـ executor
        .await?
        .ok_or(CheckoutError::UserNotFound)?;

    if balance < amount {
        // return هنا = tx بتتعملها drop = ROLLBACK
        return Err(CheckoutError::InsufficientBalance {
            have: balance,
            need: amount,
        });
    }

    sqlx::query("UPDATE users SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    let order_id = Uuid::new_v4();
    sqlx::query("INSERT INTO orders (id, user_id, total, meta) VALUES ($1, $2, $3, $4)")
        .bind(order_id)
        .bind(user_id)
        .bind(amount)
        .bind(sqlx::types::Json(
            json!({ "source": "lesson-13", "items": 2 }),
        ))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(order_id)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://app:app@localhost:5432/learn".into());

    // ═══ 1) Pool ═══
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&url)
        .await
        .context("مش قادر أتصل بـ Postgres — شغّلت docker compose up -d postgres؟")?;
    println!("✅ connected");

    // ═══ 2) Migrations ═══
    // migrate! بيضمّن ملفات migrations/ جوه الـ binary نفسه وقت الـ compile
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("✅ migrations applied");

    // ═══ 3) CRUD ═══
    let repo = UserRepo { pool: pool.clone() };
    let email = format!("user-{}@test.com", &Uuid::new_v4().to_string()[..8]);
    let user = repo.create(&email, "Ahmed", 1_000).await?;
    println!("created: {user:#?}");

    // Unique violation — نعرف نوع الـ error ونتعامل معاه (زي err.code === '23505' في pg)
    match repo.create(&email, "Dup", 0).await {
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            println!("⚠️ email موجود قبل كده (constraint: {:?})", db.constraint())
        }
        other => println!("unexpected: {other:?}"),
    }

    println!(
        "find: {:?}",
        repo.find_by_email(&email).await?.map(|u| u.name)
    );
    println!(
        "find missing: {:?}",
        repo.find_by_email("nope@x.com").await?.map(|u| u.name)
    );
    for u in repo.list(3).await? {
        println!(
            "  - {} <{}> balance={} at {}",
            u.name, u.email, u.balance, u.created_at
        );
    }

    // ═══ 4) Transactions ═══
    match checkout(&pool, user.id, 300).await {
        Ok(id) => println!("✅ order {id}"),
        Err(e) => println!("❌ {e}"),
    }
    match checkout(&pool, user.id, 5_000).await {
        Ok(id) => println!("✅ order {id}"),
        Err(e) => println!("❌ {e}  ← اتعمل rollback"),
    }

    // ═══ 5) query_scalar / Row يدوي / JSONB ═══
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM orders WHERE user_id = $1")
        .bind(user.id)
        .fetch_one(&pool)
        .await?;
    println!("orders for user = {count}");

    let row = sqlx::query("SELECT name, balance FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&pool)
        .await?;
    let name: String = row.try_get("name")?;
    let balance: i64 = row.try_get("balance")?;
    println!("manual row: {name} balance={balance}");

    let orders: Vec<OrderRow> =
        sqlx::query_as("SELECT id, total, status, meta FROM orders WHERE user_id = $1")
            .bind(user.id)
            .fetch_all(&pool)
            .await?;
    for o in &orders {
        println!(
            "  order {} total={} status={} source={}",
            o.id, o.total, o.status, o.meta.0["source"]
        );
    }

    // ═══ 6) Streaming — لو عندك مليون row متحملهمش في الـ memory مرة واحدة ═══
    use futures::TryStreamExt;
    let mut stream =
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at LIMIT 5").fetch(&pool);
    let mut n = 0;
    while let Some(u) = stream.try_next().await? {
        n += 1;
        let _ = u.id;
    }
    println!("streamed {n} users");

    // ═══ 7) Dynamic queries (filters اختيارية) — QueryBuilder بدل ما تعمل string concat ═══
    let min_balance: Option<i64> = Some(100);
    let name_like: Option<&str> = None;
    let mut qb = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM users WHERE 1=1");
    if let Some(b) = min_balance {
        qb.push(" AND balance >= ").push_bind(b);
    }
    if let Some(n) = name_like {
        qb.push(" AND name ILIKE ").push_bind(format!("%{n}%"));
    }
    qb.push(" LIMIT 5");
    println!("sql = {}", qb.sql().as_str());
    let filtered: Vec<User> = qb.build_query_as().fetch_all(&pool).await?;
    println!("filtered = {}", filtered.len());

    pool.close().await;
    Ok(())
}
