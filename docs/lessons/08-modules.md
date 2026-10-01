# 08 — Modules وتنظيم المشروع

📄 الكود: [`examples/08_modules.rs`](../examples/08_modules.rs)

## الفرق الأساسي

| Node | Rust |
|---|---|
| كل ملف = module أوتوماتيك | لازم تعلن `mod x;` في الملف الأب |
| `export` | `pub` |
| الـ default: كل حاجة private لحد ما تعمل export | نفس الكلام: private by default |
| `import { x } from './a/b'` | `use crate::a::b::x;` |
| relative `../` | `super::` |
| `index.js` | `mod.rs` أو `a.rs` جنب فولدر `a/` |

## شكل مشروع حقيقي

```
src/
├── main.rs              mod config; mod orders; mod db;
├── config.rs
├── db.rs
└── orders/
    ├── mod.rs           pub mod model; pub mod service; pub use service::OrderService;
    ├── model.rs
    └── service.rs       use crate::db::Pool; use super::model::Order;
```

## مستويات الـ visibility

| | مين يشوفه |
|---|---|
| (من غير حاجة) | الـ module ده وأولاده |
| `pub(super)` | الـ module الأب |
| `pub(crate)` | الـ crate كله (زي "internal") |
| `pub` | أي حد |

## Workspaces (monorepo)

الـ `test-proj` عندك workspace: كذا crate في repo واحد، `Cargo.lock` واحد، و`target/` واحد. زي npm/pnpm workspaces بالظبط، والـ `[workspace.dependencies]` بيوحّد النسخ.

**قاعدة تصميم:** crate لكل حدود واضحة (domain module، contracts مشتركة، infra). الـ compiler بيعمل compile لكل crate لوحده، فتقسيمها صح بيسرّع الـ builds كمان.

## جرّب بنفسك

اعمل مشروع جديد بـ `cargo new shop` وقسّمه:
- `src/models.rs` فيه `Product` و `Order`
- `src/store/mod.rs` + `src/store/memory.rs` فيه `InMemoryStore`
- `main.rs` بيستخدمهم
