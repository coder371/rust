# 01 — Cargo وأول برنامج

📄 الكود: [`examples/01_hello.rs`](../examples/01_hello.rs) — `cargo run --example 01_hello`

## الفكرة

في Node: `node index.js` بيقرا الملف وينفذه على طول. في Rust فيه خطوة **compile** الأول بتطلّع binary (ملف تنفيذي) يشتغل لوحده من غير ما تحتاج Rust على السيرفر.

```
Node:  index.js ──(V8 وقت التشغيل)──► تنفيذ
Rust:  main.rs  ──(rustc وقت الـ build)──► ./target/debug/app ──► تنفيذ
```

يعني الـ Docker image بتاعك في production ممكن تبقى `FROM scratch` أو `distroless` + ملف واحد حجمه كام MB. مفيش `node_modules`.

## Cargo.toml مقابل package.json

```toml
[package]                 # ≈ "name", "version"
name = "my-app"
edition = "2024"          # نسخة قواعد اللغة (زي "type": "module" بس أعم)

[dependencies]            # ≈ "dependencies"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]        # ≈ "devDependencies"
```

**الـ features** حاجة مش موجودة في npm: الـ crate بيقسم نفسه لأجزاء، وإنت بتفعّل اللي محتاجه بس. ده بيقلل وقت الـ compile وحجم الـ binary.

## ليه `println!` فيها علامة تعجب؟

لأنها **macro** مش function. الـ macro بيتفرد لكود عادي وقت الـ compile. ده بيسمح بحاجات مستحيلة في functions عادية زي:
- عدد arguments متغير
- التحقق وقت الـ compile إن عدد `{}` = عدد الـ values

## جرّب بنفسك

1. شيل الكومنت من `println!("{} {}", 1);` وشوف رسالة الـ error.
2. حط `;` بعد `a + b` في function `add` واقرا الـ error كويس — فيه `help` بيقولك الحل.
3. اعمل `cargo build --release` وقارن حجم `target/release/examples/01_hello` بـ `target/debug/examples/01_hello`.
