# 00 — العقلية: من Node لـ Rust

قبل ما تكتب سطر Rust، لازم تفهم **ليه** Rust مختلفة. معظم الإحباط اللي بيقابل حد جاي من Node سببه إنه بيحاول يكتب JavaScript بـ syntax بتاع Rust.

## الفرق في جملة واحدة

> **Node بتقولك:** "اكتب بسرعة، وأنا هتعامل مع الـ memory والأخطاء وقت التشغيل".
> **Rust بتقولك:** "اقنعني وقت الـ compile إن الكود صح، وأنا أضمنلك إنه مش هيقع وقت التشغيل".

الـ compiler في Rust مش عدوك. اعتبره **code reviewer صارم جداً** بيراجع كل سطر قبل ما يوصل production.

## المقارنة الكبيرة

| الموضوع | Node.js | Rust | ليه Rust اختارت كده |
|---|---|---|---|
| تشغيل الكود | interpreter + JIT (V8) | compile لـ native binary | سرعة ومفيش runtime تقيل |
| الـ Memory | Garbage Collector | Ownership (الـ compiler بيحدد إمتى تتمسح) | مفيش GC pauses، memory قليلة ومتوقعة |
| الأنواع | dynamic (TS اختياري وبيتمسح وقت التشغيل) | static وإجباري وموجود فعلاً | الغلط يطلع وقت الـ compile |
| null / undefined | موجودين في كل حتة | مش موجودين. بدالهم `Option<T>` | "الغلطة اللي تمنها مليار دولار" |
| الأخطاء | `throw` / `try-catch` مخفية | `Result<T, E>` جزء من الـ return type | مستحيل تنسى تتعامل مع error |
| الـ Mutability | كل حاجة قابلة للتعديل | immutable by default | أقل مفاجآت |
| الـ Concurrency | thread واحد + event loop | threads حقيقية + async، والـ compiler بيمنع الـ data races | استخدام كل الـ cores بأمان |
| async runtime | جوه اللغة (libuv) | مكتبة بتختارها (tokio) | اللغة متفرضش runtime على الـ embedded مثلاً |
| الـ Promises | eager (بتبدأ فوراً) | lazy (مش بتتنفذ غير لما تعمل `.await`) | zero-cost، ومفيش شغل مالوش لازمة |
| الـ OOP | classes + inheritance | structs + traits، ومفيش inheritance | composition أوضح وأأمن |
| Package manager | npm / package.json | cargo / Cargo.toml | cargo بيعمل build + test + docs + format + lint |
| الأداء | كويس | قريب من C/C++ | |

## 5 قواعد ذهبية وإنت بتتعلم

1. **اقرا رسالة الـ error كاملة.** رسايل الـ compiler في Rust من أحسن رسايل الأخطاء في أي لغة، وغالباً بتقولك الحل بالظبط (`help: consider ...`).
2. **`.clone()` مش عيب وإنت بتتعلم.** لو الـ borrow checker مضايقك، اعمل clone وكمّل. الـ optimization بعدين.
3. **ابدأ بـ `String` (owned) في الـ structs** مش `&str`. هتتجنب الـ lifetimes في الأول.
4. **استخدم `anyhow::Result` في `main` والـ scripts**، و`thiserror` في الـ modules. (الدرس 05)
5. **متحاربش الـ compiler.** لو حاجة صعبة جداً، غالباً التصميم محتاج يتغير. Rust بتدفعك لتصميم فيه ownership واضح: مين بيملك الداتا دي؟

## الأدوات اللي هتستخدمها

```bash
cargo new my-app          # npm init
cargo add serde           # npm install serde
cargo run                 # node index.js (بعد الـ build)
cargo build --release     # build بـ optimizations (أسرع 10-100 مرة من debug)
cargo test                # npm test — الـ testing مدمج في اللغة
cargo fmt                 # prettier
cargo clippy              # eslint (بس أذكى بكتير)
cargo doc --open          # docs لكل الـ dependencies على جهازك
cargo watch -x run        # nodemon  (cargo install cargo-watch)
```

## مصطلحات هتقابلها

| Rust | أقرب حاجة في Node |
|---|---|
| crate | package |
| crates.io | npmjs.com |
| trait | interface |
| struct + impl | class (من غير inheritance) |
| enum | discriminated union في TS |
| macro (`println!`, `vec!`) | مفيش — كود بيولّد كود وقت الـ compile |
| `Option<T>` | `T \| undefined` |
| `Result<T, E>` | Promise ممكن يعمل reject، بس synchronous وtyped |
| `Vec<T>` | Array |
| `HashMap<K, V>` | Map |
| `Box<dyn Trait>` | أي object بيعمل implement لـ interface |
| `Arc<Mutex<T>>` | مفيش (Node مفيهاش shared memory بين threads) |
| tokio | libuv + event loop |
| `Future` | Promise (بس lazy) |
