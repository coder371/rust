# 🦀 تعلّم Rust — من خلفية Node.js

مسار عملي: كل درس = **ملف شرح** (الفلسفة + مقارنة بـ Node + تمارين) + **ملف كود يشتغل** متشرح سطر بسطر.

## التشغيل

```bash
cd docs
cargo run --example 01_hello           # أي درس بالاسم
cargo run --example 11_async

docker compose up -d                   # Postgres + MongoDB + RabbitMQ  (للدروس 13-17)
cargo run --example 13_postgres
```

> أول `cargo run` هياخد وقت (بيعمل compile لكل الـ dependencies مرة واحدة). بعد كده سريع.
> متغيرات البيئة في `.env.example` — الأمثلة فيها defaults تشتغل مع `docker-compose.yml` على طول.

## المسار

| # | الدرس | الكود | الفكرة الأساسية |
|---|---|---|---|
| 00 | [العقلية: من Node لـ Rust](lessons/00-mindset.md) | — | ليه Rust مختلفة، وجدول المصطلحات |
| **الجزء 1: الأساسيات** ||||
| 01 | [Cargo وأول برنامج](lessons/01-cargo-and-hello.md) | `01_hello` | compile مقابل interpret، cargo مقابل npm |
| 02 | [المتغيرات والأنواع](lessons/02-variables-and-types.md) | `02_variables` | immutable by default، String مقابل &str |
| 03 | [Ownership و Borrowing](lessons/03-ownership.md) ⭐ | `03_ownership` | memory من غير GC |
| 04 | [Structs و Enums و match](lessons/04-structs-enums-match.md) | `04_structs_enums` | مفيش classes ولا null |
| 05 | [الأخطاء: Result و ?](lessons/05-errors.md) | `05_errors` | مفيش exceptions |
| 06 | [Traits و Generics](lessons/06-traits-generics.md) | `06_traits` | interfaces من غير inheritance |
| 07 | [Collections و Iterators](lessons/07-collections-iterators.md) | `07_iterators` | lazy مقابل eager |
| 08 | [Modules](lessons/08-modules.md) | `08_modules` | تنظيم المشروع |
| 09 | [Lifetimes](lessons/09-lifetimes.md) | `09_lifetimes` | متخافش منها |
| 10 | [Concurrency](lessons/10-concurrency.md) | `10_concurrency` | threads حقيقية بأمان |
| 11 | [Async/Await و Tokio](lessons/11-async-tokio.md) ⭐ | `11_async` | Futures كسولة |
| 12 | [JSON و Serde](lessons/12-serde-json.md) | `12_serde` | parse = validation |
| **الجزء 2: قواعد البيانات** ||||
| 13 | [PostgreSQL مع sqlx](lessons/13-postgres-sqlx.md) ⭐ | `13_postgres` | SQL حقيقي + type safety، transactions |
| 14 | [MongoDB](lessons/14-mongodb.md) | `14_mongodb` | typed collections بدل mongoose |
| **الجزء 3: RabbitMQ** ||||
| 15 | [RabbitMQ: الأساسيات](lessons/15-rabbitmq-basics.md) | `15_rabbit_publisher` / `15_rabbit_consumer` | ack، prefetch، confirms |
| 16 | [RabbitMQ: Patterns](lessons/16-rabbitmq-patterns.md) ⭐ | `16_rabbit_patterns` | topic، retry، DLQ، graceful shutdown |
| **الجزء 4: مشروع** ||||
| 17 | [Postgres + RabbitMQ: Outbox](lessons/17-capstone-outbox.md) | `17_capstone_outbox` | Outbox + idempotency |

## إزاي تذاكر

1. **اقرا ملف الشرح** (الفلسفة والمقارنة).
2. **افتح ملف الكود** واقراه — الكومنتات جزء أساسي من الشرح.
3. **شغّله** وشوف النتيجة.
4. **اكسره:** شيل الكومنت من الأسطر اللي عليها ❌ واقرا رسالة الـ compiler.
5. **حل تمارين "جرّب بنفسك"** — اعمل ملف جديد في `examples/` (مثلاً `examples/ex03.rs`) وشغّله بـ `cargo run --example ex03`.

## أدوات هتفرق معاك

```bash
cargo install cargo-watch     # cargo watch -x 'run --example 03_ownership'  (زي nodemon)
cargo install sqlx-cli        # sqlx migrate run / sqlx prepare
cargo clippy --examples       # linter
cargo fmt                     # formatter
```

- إضافة **rust-analyzer** في VS Code / RustRover — مهمة جداً (بتوريك الأنواع inline).
- [The Rust Book](https://doc.rust-lang.org/book/) — المرجع الرسمي.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [docs.rs](https://docs.rs) — docs أي crate.

## الهيكل

```
docs/
├── README.md               ← إنت هنا
├── Cargo.toml              ← الـ dependencies (زي package.json)
├── docker-compose.yml      ← Postgres + Mongo + RabbitMQ
├── .env.example
├── lessons/                ← الشرح (00 → 17)
├── examples/               ← الكود (كل ملف = درس يشتغل لوحده)
└── migrations/             ← SQL migrations (sqlx)
```
