//! الدرس 06 — Traits و Generics
//! شغّل:  cargo run --example 06_traits
//!
//! في TS: interface + duck typing. أي object فيه نفس الشكل "بيمشي".
//! في Rust: trait = مجموعة سلوكيات. النوع لازم يعلن صراحةً  `impl Trait for Type`.
//!          ومفيش inheritance للـ data — الـ traits هي الطريقة الوحيدة للـ polymorphism.
//!
//! الفرق الفلسفي المهم: الـ generics في TS بتتمسح وقت التشغيل (type erasure).
//! في Rust الـ generics بتتعمل لها نسخة لكل نوع وقت الـ compile (monomorphization)
//! → zero-cost: نفس سرعة إنك تكتب الكود يدوي لكل نوع.

use std::fmt;

// ═══ تعريف trait ═══
trait Notifier {
    fn channel(&self) -> &str;
    fn send(&self, to: &str, msg: &str) -> Result<(), String>;

    // default method — زي abstract class فيها implementation
    fn send_welcome(&self, to: &str) -> Result<(), String> {
        self.send(to, "أهلاً بيك 👋")
    }
}

struct Email {
    from: String,
}
struct Sms;

impl Notifier for Email {
    fn channel(&self) -> &str {
        "email"
    }
    fn send(&self, to: &str, msg: &str) -> Result<(), String> {
        println!("  [email from {}] → {to}: {msg}", self.from);
        Ok(())
    }
}

impl Notifier for Sms {
    fn channel(&self) -> &str {
        "sms"
    }
    fn send(&self, to: &str, msg: &str) -> Result<(), String> {
        if !to.starts_with("01") {
            return Err(format!("رقم غلط {to}"));
        }
        println!("  [sms] → {to}: {msg}");
        Ok(())
    }
}

// ═══ 3 طرق تستخدم بيها trait ═══

// (أ) Generic — static dispatch (الأسرع، الـ compiler بيعمل نسخة لكل نوع)
fn notify_generic<N: Notifier>(n: &N, to: &str) {
    let _ = n.send_welcome(to);
}

// (ب) impl Trait — نفس (أ) بكتابة أقصر
fn notify_impl(n: &impl Notifier, to: &str) {
    let _ = n.send_welcome(to);
}

// (ج) dyn Trait — dynamic dispatch (زي JS: vtable وقت التشغيل)
//     لازم لما تحط أنواع مختلفة في نفس الـ Vec
fn notify_all(list: &[Box<dyn Notifier>], to: &str) {
    for n in list {
        match n.send(to, "عرض خاص") {
            Ok(()) => println!("  ✅ via {}", n.channel()),
            Err(e) => println!("  ❌ via {}: {e}", n.channel()),
        }
    }
}

// ═══ Generics مع constraints ═══
// TS: function largest<T extends Comparable>(items: T[]): T
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter().copied();
    let mut max = iter.next()?;
    for x in iter {
        if x > max {
            max = x
        }
    }
    Some(max)
}

// where clause لما الـ constraints تكتر
fn print_all<T>(items: &[T])
where
    T: fmt::Display,
{
    for i in items {
        print!("[{i}] ")
    }
    println!();
}

// ═══ Generic struct ═══
#[derive(Debug)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }
    fn err(msg: &str) -> Self {
        Self {
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ═══ الـ traits المشهورة من الـ standard library ═══
struct Money {
    cents: i64,
}

// Display = toString() — بيخلي {} تشتغل
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02} EGP", self.cents / 100, self.cents % 100)
    }
}

// From = conversions. لو عملت From، بتاخد Into ببلاش
impl From<f64> for Money {
    fn from(v: f64) -> Self {
        Money {
            cents: (v * 100.0).round() as i64,
        }
    }
}

// operator overloading = implement trait
impl std::ops::Add for Money {
    type Output = Money;
    fn add(self, o: Money) -> Money {
        Money {
            cents: self.cents + o.cents,
        }
    }
}

fn main() {
    println!("── static dispatch ──");
    let email = Email {
        from: "noreply@shop.com".into(),
    };
    notify_generic(&email, "a@x.com");
    notify_impl(&Sms, "01000000000");

    println!("── dynamic dispatch ──");
    let channels: Vec<Box<dyn Notifier>> = vec![Box::new(email), Box::new(Sms)];
    notify_all(&channels, "a@x.com");

    println!("── generics ──");
    println!("{:?}", largest(&[3, 9, 2]));
    println!("{:?}", largest(&[1.5, 0.2]));
    println!("{:?}", largest::<i32>(&[]));
    print_all(&["a", "b"]);
    let res = ApiResponse::ok(vec![1, 2]);
    println!(
        "{res:?} → success={}",
        res.data.is_some() && res.error.is_none()
    );
    println!("{:?}", ApiResponse::<()>::err("boom"));

    println!("── std traits ──");
    let total = Money::from(10.5) + 2.25.into();
    println!("total = {total}");
    let s: String = total.to_string(); // ToString بتيجي ببلاش مع Display
    println!("{s}");
}
