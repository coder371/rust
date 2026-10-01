# 13 — PostgreSQL مع sqlx

📄 الكود: [`examples/13_postgres.rs`](../examples/13_postgres.rs) · Migrations: [`migrations/`](../migrations/)

```bash
docker compose up -d postgres
cargo run --example 13_postgres
```

## الخريطة: إيه المقابل لإيه

| Node | Rust | الأسلوب |
|---|---|---|
| `pg` | `tokio-postgres` | driver خام |
| `pg` + types يدوي | **`sqlx`** ⭐ | SQL حقيقي + type safety |
| `knex` / `kysely` | `sea-query` | query builder |
| Prisma / TypeORM / Sequelize | `SeaORM` / `Diesel` | ORM |

**ليه sqlx هو الاختيار الشائع؟** فلسفة Rust بتميل لـ "اكتب SQL حقيقي". الـ ORMs الكبيرة بتخبّي الـ SQL وبتعمل N+1 queries من غير ما تاخد بالك. sqlx بيديك SQL كامل + ضمانات الأنواع.

## runtime check مقابل compile-time check

```rust
// runtime: الغلط يطلع لما تشغّل (اللي في الدرس)
sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")

// compile-time: الـ macro بيكلم الـ DB وقت الـ build ويتأكد من كل حاجة 🤯
sqlx::query_as!(User, "SELECT id, email FROM userz WHERE id = $1", id)
//                                            ^^^^^ error: relation "userz" does not exist
```

الـ macros محتاجة `DATABASE_URL` وقت الـ build. للـ CI: `cargo sqlx prepare` بيحفظ الـ metadata في `.sqlx/` وتعمل commit ليها (offline mode). (`cargo install sqlx-cli`)

## الـ concepts الأساسية

| المفهوم | Node (pg) | Rust (sqlx) |
|---|---|---|
| Pool | `new Pool({ max: 10 })` | `PgPoolOptions::new().max_connections(10).connect(url)` |
| مشاركة الـ Pool | module singleton | `pool.clone()` (رخيص — Arc جواه) |
| Query | `pool.query(sql, [a, b])` | `sqlx::query(sql).bind(a).bind(b).execute(&pool)` |
| صف واحد | `rows[0]` | `.fetch_one()` (error لو مفيش) |
| صف أو مفيش | `rows[0] ?? null` | `.fetch_optional()` → `Option<T>` |
| كل الصفوف | `rows` | `.fetch_all()` → `Vec<T>` |
| stream | `pg-query-stream` | `.fetch()` → `Stream` |
| قيمة واحدة | `rows[0].count` | `query_scalar` |
| Mapping | يدوي | `#[derive(FromRow)]` |
| Transaction | `BEGIN` / `COMMIT` / `ROLLBACK` يدوي | `pool.begin()` → `tx.commit()`، والـ rollback أوتوماتيك لو حصل drop |
| Unique violation | `err.code === '23505'` | `db_err.is_unique_violation()` |
| Migrations | knex / prisma migrate | `sqlx::migrate!()` أو `sqlx migrate run` |

## Transaction + RAII = أمان ببلاش

```js
// Node — لازم تفتكر كل حاجة
const client = await pool.connect();
try {
  await client.query('BEGIN');
  ...
  await client.query('COMMIT');
} catch (e) {
  await client.query('ROLLBACK');
  throw e;
} finally {
  client.release();
}
```
```rust
// Rust — مستحيل تنسى
let mut tx = pool.begin().await?;
sqlx::query("...").execute(&mut *tx).await?;   // لو فشلت → return → tx drop → ROLLBACK
sqlx::query("...").execute(&mut *tx).await?;
tx.commit().await?;                             // الـ connection بترجع الـ pool لوحدها
```

## SQL Injection: الـ API نفسه بيحميك

في sqlx 0.9، `sqlx::query()` بتقبل `&'static str` بس. يعني:
```rust
sqlx::query(&format!("SELECT * FROM users WHERE name = '{name}'"))  // ❌ مش هيعمل compile
```
للـ queries الديناميكية (filters اختيارية) استخدم `QueryBuilder` مع `push_bind`.

## نصايح

- **الفلوس بـ `BIGINT` بالقروش**، مش `FLOAT`.
- **`FOR UPDATE`** لما تقرا حاجة هتعدّل عليها في نفس الـ transaction (رصيد، stock).
- **Repository pattern**: struct ماسك الـ pool وfunctions typed. ده بديل الـ "Model" في الـ ORMs.

## جرّب بنفسك

1. زوّد migration جديدة `20260923000002_products.sql` فيها جدول `products (id, sku UNIQUE, name, price, stock)`.
2. اعمل `ProductRepo` بـ `create`, `find_by_sku`, `list_in_stock`, `decrement_stock(sku, qty)` — الأخيرة لازم تكون atomic (`UPDATE ... SET stock = stock - $1 WHERE sku = $2 AND stock >= $1` وشوف `rows_affected()`).
3. اعمل `checkout` بتخصم الرصيد + الـ stock + تعمل order — كلهم في transaction واحدة.
4. (متقدم) `cargo install sqlx-cli` وحوّل query واحدة لـ `query_as!` وشوف الـ compile-time checking.
