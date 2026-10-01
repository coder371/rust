//! الدرس 08 — Modules و Visibility
//! شغّل:  cargo run --example 08_modules
//!
//! في Node: كل ملف = module، وبتعمل export للي عايزه، وimport من path.
//! في Rust: الـ modules شجرة (tree) بتبدأ من main.rs أو lib.rs.
//!   - `mod x;`         → "فيه module اسمه x" (الـ compiler يدور على x.rs أو x/mod.rs)
//!   - كل حاجة private by default — لازم `pub` عشان تطلع برّه
//!   - `use` = import (بس ده مجرد اختصار للاسم، مش تحميل ملف)
//!
//! الشكل في مشروع حقيقي:
//!   src/
//!   ├── main.rs          ← mod config; mod orders;
//!   ├── config.rs        ← pub struct Config {...}
//!   └── orders/
//!       ├── mod.rs       ← pub mod model; pub mod service;   (أو orders.rs جنب الفولدر)
//!       ├── model.rs
//!       └── service.rs   ← use crate::config::Config;   use super::model::Order;
//!
//! هنا هنعمل الـ modules جوه نفس الملف بـ `mod name { ... }` عشان المثال يبقى ملف واحد.

mod config {
    #[derive(Debug)]
    pub struct Config {
        pub db_url: String, // pub field → يتقري من برّه
        secret: String,     // private → محدش برّه الـ module يشوفه
    }

    impl Config {
        pub fn from_env() -> Self {
            Self {
                db_url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://localhost".into()),
                secret: "s3cr3t".into(),
            }
        }
        pub fn masked_secret(&self) -> String {
            format!("{}***", &self.secret[..2])
        }
    }
}

mod orders {
    // sub-modules
    pub mod model {
        #[derive(Debug)]
        pub struct Order {
            pub id: u32,
            pub total: f64,
        }
    }

    pub mod service {
        use super::model::Order; // super = الـ module الأب  (زي ../)
        use crate::config::Config; // crate = الـ root         (زي absolute path)

        pub fn create(cfg: &Config, total: f64) -> Order {
            println!("  saving to {}", cfg.db_url);
            helpers::log("created");
            Order { id: 1, total }
        }

        mod helpers {
            // private: متاح جوه service بس
            pub(crate) fn log(msg: &str) {
                // pub(crate) = public جوه الـ crate بس
                println!("  [log] {msg}");
            }
        }
    }

    // re-export — زي  export { create } from './service'
    pub use service::create;
}

// use بيختصر الأسماء. ينفع تعمل alias بـ as (زي import { x as y })
use orders::model::Order as OrderModel;

fn main() {
    let cfg = config::Config::from_env();
    println!("{:?}", cfg.db_url);
    // println!("{}", cfg.secret); // ❌ field `secret` is private
    println!("secret = {}", cfg.masked_secret());

    let o: OrderModel = orders::create(&cfg, 99.5);
    println!("{o:?} → id={} total={}", o.id, o.total);

    // الـ crates الخارجية (زي npm packages) بتتضاف في Cargo.toml وتستخدمها بـ use على طول:
    let id = uuid::Uuid::new_v4();
    println!("uuid from external crate: {id}");
}

// ملاحظات مقارنة بـ npm:
// - Cargo.toml   ≈ package.json          | Cargo.lock ≈ package-lock.json
// - cargo add x  ≈ npm install x          | crates.io  ≈ npmjs.com   | docs.rs = docs لكل crate أوتوماتيك
// - features     = حاجة مش موجودة في npm: بتفعّل أجزاء من الـ crate بس (زي tokio features = ["full"])
// - workspace    ≈ npm workspaces / monorepo (زي test-proj عندك)
// - مفيش node_modules لكل مشروع: الـ source بيتحمل مرة في ~/.cargo/registry والـ build في target/
