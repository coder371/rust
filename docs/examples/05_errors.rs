//! الدرس 05 — الـ Errors: Result و Option و `?`
//! شغّل:  cargo run --example 05_errors
//!
//! في Node: أي function ممكن تعمل throw، ومفيش حاجة في الـ signature بتقولك كده.
//!          فبتنسى الـ try/catch، أو بتمسك error مش عارف نوعه (catch (e: unknown)).
//! في Rust: مفيش exceptions. الـ error جزء من الـ return type:
//!          fn parse(s: &str) -> Result<Config, ParseError>
//!          الـ compiler بيجبرك تتعامل معاه (أو تعدّيه لفوق صراحةً بـ `?`).
//!
//! نوعين من الأخطاء:
//!   - recoverable  → Result<T, E>     (ملف مش موجود، input غلط، DB وقعت)
//!   - bugs         → panic!           (index برّه الـ array، invariant اتكسر) — زي crash

use std::collections::HashMap;
use std::fmt;

// ════════ 1) Result الأساسي ════════
fn parse_age(input: &str) -> Result<u8, String> {
    let n: i64 = input
        .trim()
        .parse()
        .map_err(|_| format!("'{input}' مش رقم"))?;
    if !(0..=150).contains(&n) {
        return Err(format!("{n} عمر مش منطقي"));
    }
    Ok(n as u8)
}

// ════════ 2) الـ `?` operator ════════
// `?` معناها: لو Ok خد القيمة وكمّل، لو Err ارجع بيه فوراً من الـ function.
// ده بالظبط زي `await` لما الـ promise بيعمل reject والـ error بيطلع لفوق —
// بس هنا ظاهر في الكود وفي الـ type.
fn sum_ages(a: &str, b: &str) -> Result<u16, String> {
    let x = parse_age(a)?;
    let y = parse_age(b)?;
    Ok(x as u16 + y as u16)
}

// ════════ 3) Custom error types (كتابة يدوي) ════════
#[derive(Debug)]
enum ConfigError {
    Missing(String),
    Invalid { key: String, value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Missing(k) => write!(f, "missing key: {k}"),
            ConfigError::Invalid { key, value } => write!(f, "invalid {key}={value}"),
        }
    }
}
impl std::error::Error for ConfigError {}

// ════════ 4) نفس الكلام بـ thiserror (ده اللي هتستخدمه في الحقيقة — للـ libraries/modules) ════════
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("user {0} not found")]
    NotFound(u64),
    #[error("validation failed: {0}")]
    Validation(String),
    // #[from] بيولّد impl From<ConfigError> → يخلي `?` يحوّل تلقائي
    #[error("config: {0}")]
    Config(#[from] ConfigError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

fn get_port(env: &HashMap<&str, &str>) -> Result<u16, AppError> {
    let raw = env
        .get("PORT")
        .ok_or_else(|| ConfigError::Missing("PORT".into()))?; // ConfigError → AppError تلقائي
    let port: u16 = raw.parse()?; // ParseIntError → AppError تلقائي
    if port < 1024 {
        return Err(AppError::Validation(format!("port {port} محجوز")));
    }
    Ok(port)
}

fn find_user(id: u64) -> Result<&'static str, AppError> {
    if id == 1 {
        Ok("Ahmed")
    } else {
        Err(AppError::NotFound(id))
    }
}

// ════════ 5) anyhow (للـ applications: main, scripts, handlers) ════════
// القاعدة المشهورة:
//   - library/module → thiserror (errors ليها أنواع، اللي بيناديك يقدر يعمل match)
//   - application    → anyhow   (أي error، مع context، مش مهم النوع)
use anyhow::{Context, bail};

fn load_settings(path: &str) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read settings from {path}"))?;
    if content.is_empty() {
        bail!("settings file is empty"); // = return Err(anyhow!(...))
    }
    Ok(content)
}

fn main() {
    println!("── Result ──");
    for input in ["25", "abc", "200"] {
        match parse_age(input) {
            Ok(age) => println!("✅ {age}"),
            Err(e) => println!("❌ {e}"),
        }
    }
    println!("sum = {:?}", sum_ages("20", "30"));
    println!("sum = {:?}", sum_ages("20", "x"));

    println!("── custom errors ──");
    let e = ConfigError::Invalid {
        key: "PORT".into(),
        value: "abc".into(),
    };
    println!("{e}   (debug: {e:?})");

    let mut env = HashMap::new();
    println!("{:?}", get_port(&env).map_err(|e| e.to_string()));
    env.insert("PORT", "abc");
    println!("{:?}", get_port(&env).map_err(|e| e.to_string()));
    env.insert("PORT", "80");
    println!("{:?}", get_port(&env).map_err(|e| e.to_string()));
    env.insert("PORT", "8080");
    println!("{:?}", get_port(&env).map_err(|e| e.to_string()));

    // match على نوع الـ error — ده اللي مستحيل تعمله بأمان في JS
    match find_user(7) {
        Ok(u) => println!("user {u}"),
        Err(AppError::NotFound(id)) => println!("→ 404 for {id}"),
        Err(other) => println!("→ 500 {other}"),
    }

    println!("── anyhow + context ──");
    if let Err(e) = load_settings("/nope/settings.toml") {
        // {:#} بيطبع السلسلة كلها: الـ context + السبب الأصلي
        println!("{e:#}");
    }

    println!("── Option helpers ──");
    let maybe: Option<i32> = Some(4);
    println!("{:?}", maybe.map(|x| x * 2)); // Some(8)
    println!("{:?}", maybe.filter(|x| *x > 10)); // None
    println!("{}", maybe.filter(|x| *x > 10).unwrap_or_default()); // 0
    let as_result: Result<i32, &str> = None.ok_or("empty"); // Option → Result
    println!("{as_result:?}");

    println!("── unwrap / expect / panic ──");
    // unwrap = "أنا متأكد إن ده مش error، ولو طلع error اعمل crash"
    // استخدمه في الـ tests والـ prototypes. في production استخدم expect برسالة واضحة.
    let n: i32 = "42".parse().expect("hardcoded number must parse");
    println!("n = {n}");
    // let boom: i32 = "x".parse().unwrap(); // 💥 panic: called `Result::unwrap()` on an `Err` value
}
