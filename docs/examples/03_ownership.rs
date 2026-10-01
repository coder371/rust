//! الدرس 03 — Ownership و Borrowing  (أهم درس في Rust كلها)
//! شغّل:  cargo run --example 03_ownership
//!
//! في Node: الـ Garbage Collector بيلف كل شوية ويشوف مين محدش بيشاور عليه ويمسحه.
//!          ده مريح، لكن تمنه: pauses، memory أكتر، ومفيش حد يمنعك إن حتتين في الكود
//!          يعدّلوا نفس الـ object في نفس الوقت (shared mutable state = bugs).
//! في Rust: مفيش GC. الـ compiler نفسه بيعرف إمتى كل قيمة تتمسح، وده بـ 3 قواعد:
//!   1) كل قيمة ليها owner واحد بس.
//!   2) لما الـ owner يخرج من الـ scope، القيمة بتتمسح فوراً (drop).
//!   3) ينفع تستلف (borrow) القيمة: إما  &T  كتير (قراية)  أو  &mut T  واحد بس (كتابة) — مش الاتنين مع بعض.

fn main() {
    move_semantics();
    copy_types();
    borrowing();
    mutable_borrowing();
    slices();
    drop_order();
}

fn move_semantics() {
    println!("── move ──");
    let a = String::from("hello");
    let b = a; // ← الـ ownership "اتنقل" لـ b. الـ a بقت invalid.
    // println!("{a}"); // ❌ error: borrow of moved value: `a`
    println!("b = {b}");

    // في JS:  let a = {x:1}; let b = a;  → الاتنين بيشاوروا على نفس الـ object.
    // في Rust مينفعش يبقى فيه اتنين "مالكين"، فالـ compiler بيعتبر a خلاص انتهت.
    // لو عايز نسخة حقيقية (deep copy) لازم تقول كده صراحةً:
    let c = b.clone();
    println!("b = {b}, c = {c}");

    // نفس الكلام لما تبعت قيمة لـ function:
    let s = String::from("owned");
    takes_ownership(s);
    // println!("{s}"); // ❌ s اتنقلت جوه الـ function واتمسحت لما خلصت
}

fn takes_ownership(s: String) {
    println!("got {s}");
} // ← هنا s بتتمسح من الـ memory (drop). مفيش GC — ده deterministic.

fn copy_types() {
    println!("── Copy ──");
    // الأنواع الصغيرة اللي على الـ stack (أرقام، bool، char، tuples منهم) بتتنسخ تلقائي
    let x = 5;
    let y = x; // نسخة، مش move
    println!("x={x} y={y}"); // الاتنين شغالين
}

fn borrowing() {
    println!("── borrow (&) ──");
    let name = String::from("Rust");
    // بدل ما نعطي الـ ownership، بنعطي "reference" — استلاف للقراية
    let len = length(&name);
    println!("{name} has length {len}"); // name لسه بتاعتنا ✅

    // ينفع يكون فيه أي عدد من الـ & في نفس الوقت
    let r1 = &name;
    let r2 = &name;
    println!("{r1} {r2}");
}

fn length(s: &str) -> usize {
    // &String بتتحول تلقائي لـ &str (deref coercion) — عشان كده &str أعم في الـ params
    s.len()
}

fn mutable_borrowing() {
    println!("── borrow (&mut) ──");
    let mut list = vec![1, 2, 3];
    add_item(&mut list);
    println!("{list:?}");

    // القاعدة: &mut واحد بس في نفس الوقت، ومفيش & معاه.
    // ليه؟ ده بيمنع data races وقت الـ compile + بيمنع bug زي ده:
    let first = &list[0];
    // list.push(4); // ❌ لو الـ Vec كبر واتنقل في الـ memory، first هيشاور على memory ميتة!
    println!("first = {first}");
    list.push(4); // ✅ هنا عادي لأن first مبقاش مستخدم بعد كده (NLL)
    println!("{list:?}");

    // في JS نفس الـ bug بشكل تاني: تعدّل array وإنت بتلف عليه بـ for...of
    // Rust بتمنعه:
    // for x in &list { list.push(*x); } // ❌
}

fn add_item(v: &mut Vec<i32>) {
    v.push(99);
}

fn slices() {
    println!("── slices ──");
    let text = String::from("hello world");
    let word = first_word(&text);
    // text.clear(); // ❌ مينفعش تمسح النص وفيه slice بيشاور عليه
    println!("first word = {word}");

    let nums = [10, 20, 30, 40];
    let middle: &[i32] = &nums[1..3];
    println!("middle = {middle:?}");
}

fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

struct Connection(&'static str);

impl Drop for Connection {
    // زي destructor — بيتنفذ أوتوماتيك لما القيمة تخرج من الـ scope.
    // ده اسمه RAII: الـ connection/file/lock بيتقفل لوحده. مفيش `finally { conn.close() }`.
    fn drop(&mut self) {
        println!("closing {}", self.0);
    }
}

fn drop_order() {
    println!("── drop / RAII ──");
    let _a = Connection("A");
    {
        let _b = Connection("B");
        println!("inner scope ending");
    } // ← B اتقفلت هنا
    println!("outer scope ending");
} // ← A اتقفلت هنا
