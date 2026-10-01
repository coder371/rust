//! الدرس 07 — Collections و Iterators و Closures
//! شغّل:  cargo run --example 07_iterators
//!
//! في JS:   arr.filter(...).map(...)  → كل خطوة بتعمل array جديد في الـ memory (eager).
//! في Rust: iter().filter().map()     → lazy! مفيش حاجة بتتنفذ لحد ما تعمل collect/sum/for.
//!          والـ compiler بيحوّل السلسلة كلها لـ loop واحد (zero-cost abstraction).

use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
struct Order {
    id: u32,
    customer: String,
    total: f64,
    paid: bool,
}

fn orders() -> Vec<Order> {
    vec![
        Order {
            id: 1,
            customer: "ahmed".into(),
            total: 120.0,
            paid: true,
        },
        Order {
            id: 2,
            customer: "sara".into(),
            total: 80.0,
            paid: false,
        },
        Order {
            id: 3,
            customer: "ahmed".into(),
            total: 300.0,
            paid: true,
        },
        Order {
            id: 4,
            customer: "omar".into(),
            total: 50.0,
            paid: true,
        },
    ]
}

fn main() {
    let list = orders();

    // ═══ 3 طرق تلف على Vec ═══
    // .iter()      → &T      (استلاف — الـ list تفضل بتاعتك)
    // .iter_mut()  → &mut T  (تعدّل في مكانها)
    // .into_iter() → T       (تستهلك الـ list — بتاخد ownership العناصر)
    // و `for x in &list` = `for x in list.iter()`

    // JS: list.filter(o => o.paid).map(o => o.total).reduce((a, b) => a + b, 0)
    let paid_total: f64 = list.iter().filter(|o| o.paid).map(|o| o.total).sum();
    println!("paid total = {paid_total}");

    // JS: list.map(o => o.id)
    let ids: Vec<u32> = list.iter().map(|o| o.id).collect();
    println!("ids = {ids:?}");

    // find / some / every / findIndex
    println!(
        "find  = {:?}",
        list.iter().find(|o| o.total > 100.0).map(|o| o.id)
    );
    println!("some  = {}", list.iter().any(|o| !o.paid));
    println!("every = {}", list.iter().all(|o| o.total > 0.0));
    println!(
        "index = {:?}",
        list.iter().position(|o| o.customer == "omar")
    );

    // sort — بيعدّل في مكانه (زي JS) لكن لازم mut
    let mut sorted = list.clone();
    sorted.sort_by(|a, b| b.total.total_cmp(&a.total)); // تنازلي
    println!(
        "top = {:?}",
        sorted.iter().take(2).map(|o| o.id).collect::<Vec<_>>()
    );

    // الـ lazy بشكل عملي: مفيش حاجة هتطبع لحد الـ collect
    let lazy = list.iter().map(|o| {
        println!("  processing {}", o.id);
        o.id * 10
    });
    println!("قبل الـ collect (مفيش حاجة اتنفذت)");
    let first_two: Vec<u32> = lazy.take(2).collect(); // وهيعالج اتنين بس!
    println!("first_two = {first_two:?}");

    // enumerate / zip / chain / rev / windows / chunks
    for (i, o) in list.iter().enumerate().skip(2) {
        println!("  {i}: {}", o.customer)
    }
    let pairs: Vec<(u32, char)> = ids.iter().copied().zip("abcd".chars()).collect();
    println!("zip = {pairs:?}");
    println!("windows = {:?}", ids.windows(2).collect::<Vec<_>>());
    println!("chunks  = {:?}", ids.chunks(3).collect::<Vec<_>>());

    // collect ذكي: ممكن يجمع Result — لو أي عنصر Err، الكل يبقى Err
    let parsed: Result<Vec<i32>, _> = ["1", "2", "3"].iter().map(|s| s.parse::<i32>()).collect();
    let failed: Result<Vec<i32>, _> = ["1", "x"].iter().map(|s| s.parse::<i32>()).collect();
    println!("parsed = {parsed:?}, failed = {}", failed.is_err());

    // ═══ HashMap (زي Map في JS / object as dictionary) ═══
    // groupBy customer → sum
    let mut per_customer: HashMap<String, f64> = HashMap::new();
    for o in &list {
        // entry API: بديل  map[k] = (map[k] ?? 0) + v
        *per_customer.entry(o.customer.clone()).or_insert(0.0) += o.total;
    }
    println!("per customer = {per_customer:?}"); // الترتيب مش ثابت!

    // BTreeMap = مترتب بالـ key
    let sorted_map: BTreeMap<_, _> = per_customer.iter().collect();
    println!("sorted = {sorted_map:?}");

    match per_customer.get("ahmed") {
        Some(v) => println!("ahmed = {v}"),
        None => println!("no ahmed"),
    }

    // HashSet = Set
    let customers: HashSet<&str> = list.iter().map(|o| o.customer.as_str()).collect();
    println!("unique customers = {}", customers.len());

    // ═══ Closures ═══
    // زي arrow functions، والـ compiler بيعرف لوحده هي بتستلف ولا بتاخد ownership
    let tax = 0.14;
    let with_tax = |x: f64| x * (1.0 + tax); // بتستلف tax
    println!("with tax = {:.2}", with_tax(100.0));

    let mut calls = 0;
    let mut count = || calls += 1; // بتستلف calls كـ &mut
    count();
    count();
    println!("calls = {calls}");

    // move = خد ownership (مهم جداً مع threads و async — هتشوفه في 10 و 11)
    let name = String::from("closure");
    let owned = move || println!("I own {name}");
    owned();

    // function بتاخد closure: Fn / FnMut / FnOnce
    println!("apply = {}", apply_twice(|x| x + 3, 10));
}

fn apply_twice(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}
