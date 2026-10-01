//! الدرس 04 — Structs و Enums و Pattern Matching
//! شغّل:  cargo run --example 04_structs_enums
//!
//! في Node/TS: عندك objects و classes و interfaces و discriminated unions.
//! في Rust: struct = الداتا، impl = الـ methods، enum = "واحد من دول" (أقوى بكتير من TS enums).
//! مفيش inheritance خالص — بدالها composition + traits (الدرس 06).

// #[derive] بيولّد كود جاهز: Debug للطباعة، Clone للنسخ، PartialEq للمقارنة بـ ==
#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

// الـ methods منفصلة عن الـ data
impl User {
    // "associated function" (زي static method). Rust مفيهاش constructors — Convention: new
    fn new(id: u64, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
            active: true,
        }
    }

    // &self = بيقرا بس (زي getter)
    fn display_name(&self) -> String {
        format!("#{} {}", self.id, self.name)
    }

    // &mut self = بيعدّل
    fn deactivate(&mut self) {
        self.active = false;
    }

    // self (من غير &) = بياخد الـ ownership ويستهلك الـ object (مفيد في builders)
    fn into_email(self) -> String {
        self.email
    }
}

// ───── Enum بـ data: ده اللي TS بتسميه discriminated union ─────
// TS:
//   type PaymentMethod =
//     | { kind: "cash" }
//     | { kind: "card"; last4: string }
//     | { kind: "wallet"; provider: string; phone: string }
#[derive(Debug)]
enum PaymentMethod {
    Cash,
    Card { last4: String },
    Wallet { provider: String, phone: String },
}

#[derive(Debug)]
enum OrderStatus {
    Pending,
    Paid(PaymentMethod), // tuple variant
    Shipped { tracking: String },
    Cancelled(String),
}

fn describe(status: &OrderStatus) -> String {
    // match لازم يكون exhaustive — لو زودت variant جديد ونسيت تعالجه، مش هيعمل compile!
    // في TS لازم تعمل trick بـ `never` عشان تاخد نفس الضمان.
    match status {
        OrderStatus::Pending => "مستني الدفع".into(),
        OrderStatus::Paid(PaymentMethod::Cash) => "اتدفع كاش".into(),
        OrderStatus::Paid(PaymentMethod::Card { last4 }) => format!("كارت ****{last4}"),
        OrderStatus::Paid(PaymentMethod::Wallet { provider, phone }) => {
            format!("محفظة {provider} ({phone})")
        }
        OrderStatus::Shipped { tracking } if tracking.is_empty() => {
            "اتشحن (من غير tracking)".into()
        }
        OrderStatus::Shipped { tracking } => format!("اتشحن: {tracking}"),
        OrderStatus::Cancelled(reason) => format!("اتلغى: {reason}"),
    }
}

fn main() {
    let mut user = User::new(1, "Ahmed", "a@x.com");
    println!("{}", user.display_name());
    user.deactivate();
    println!("{user:?}");

    // struct update syntax — زي {...user, name: "Ali"}
    let other = User {
        id: 2,
        name: "Ali".into(),
        ..user.clone()
    };
    println!("{other:#?}");
    println!("equal? {}", user == other);

    let email = user.into_email(); // user اتستهلك
    println!("email: {email}");
    // println!("{user:?}"); // ❌ moved

    let statuses = vec![
        OrderStatus::Pending,
        OrderStatus::Paid(PaymentMethod::Card {
            last4: "4242".into(),
        }),
        OrderStatus::Paid(PaymentMethod::Wallet {
            provider: "Vodafone".into(),
            phone: "010".into(),
        }),
        OrderStatus::Paid(PaymentMethod::Cash),
        OrderStatus::Shipped {
            tracking: "EG123".into(),
        },
        OrderStatus::Cancelled("العميل غير رأيه".into()),
    ];
    for s in &statuses {
        println!("{:<60} → {}", format!("{s:?}"), describe(s));
    }

    // ───── Option: مفيش null و undefined ─────
    // Option<T> ده مجرد enum:  enum Option<T> { Some(T), None }
    let found = find_user(1);
    match found {
        Some(name) => println!("found {name}"),
        None => println!("not found"),
    }
    // if let = لما يهمك حالة واحدة بس
    if let Some(name) = find_user(2) {
        println!("found {name}");
    }
    // زي `user?.name ?? "guest"` في JS:
    let name = find_user(99).unwrap_or("guest");
    println!("name = {name}");

    // ───── Destructuring و match على قيم ─────
    let point = (3, -2);
    match point {
        (0, 0) => println!("origin"),
        (x, 0) | (0, x) => println!("على محور، {x}"),
        (x, y) if x > 0 && y < 0 => println!("الربع الرابع ({x}, {y})"),
        _ => println!("مكان تاني"), // _ = default
    }
    let code = 404;
    let kind = match code {
        200..=299 => "success",
        400 | 404 => "client error",
        500.. => "server error",
        _ => "other",
    };
    println!("{code} → {kind}");
}

fn find_user(id: u64) -> Option<&'static str> {
    match id {
        1 => Some("Ahmed"),
        2 => Some("Sara"),
        _ => None,
    }
}
