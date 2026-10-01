//! الدرس 09 — Lifetimes
//! شغّل:  cargo run --example 09_lifetimes
//!
//! الـ lifetime مش حاجة بتتحكم فيها وقت التشغيل — ده مجرد "label" بتقول بيه للـ compiler:
//! "الـ reference ده عايش قد إيه بالنسبة للتاني". الهدف: يمنع dangling references
//! (reference بيشاور على memory اتمسحت). في JS ده مستحيل يحصل لأن الـ GC مش هيمسح حاجة
//! لسه حد بيشاور عليها — Rust بتحقق نفس الأمان من غير GC، والتمن هو الـ annotations دي.
//!
//! الخبر الحلو: 90% من الوقت الـ compiler بيستنتجها لوحده (lifetime elision).
//! هتحتاج تكتبها بس لما function ترجع reference وعندها أكتر من reference داخل.

// ❌ من غير 'a الـ compiler مش عارف: الـ return جاي من a ولا b؟ فمش عارف يتأكد إنه عايش.
// ✅ 'a معناها: الناتج عايش قد "أقصر" واحد في a و b.
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

// struct بيمسك reference — لازم lifetime: "الـ Parser ميعيشش أطول من الـ input"
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    // بيرجع slices من الـ input الأصلي من غير أي نسخ (zero-copy) — ده سر سرعة Rust في الـ parsing
    fn next_token(&mut self) -> Option<&'a str> {
        let rest = self.input[self.pos..].trim_start();
        if rest.is_empty() {
            return None;
        }
        let start = self.input.len() - rest.len();
        let end = rest
            .find(' ')
            .map(|i| start + i)
            .unwrap_or(self.input.len());
        self.pos = end;
        Some(&self.input[start..end])
    }
}

// 'static = عايش طول عمر البرنامج (زي string literals)
fn app_name() -> &'static str {
    "rust-learn"
}

fn main() {
    let a = String::from("long string");
    let result;
    {
        let b = String::from("short");
        result = longest(&a, &b);
        println!("longest = {result}");
    }
    // println!("{result}"); // ❌ b اتمسحت، و result ممكن يكون بيشاور عليها

    let text = String::from("GET /orders HTTP/1.1");
    let mut p = Parser::new(&text);
    while let Some(tok) = p.next_token() {
        println!("token: {tok}");
    }
    println!("{}", app_name());

    // نصيحة عملية للمبتدئ: لو الـ lifetimes بقت معقدة في struct،
    // خلي الـ field String (owned) بدل &str وخلاص. clone مش عيب وإنت بتتعلم.
}
