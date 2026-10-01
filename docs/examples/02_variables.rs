//! الدرس 02 — المتغيرات والأنواع
//! شغّل:  cargo run --example 02_variables

fn main() {
    // ───────── let vs const في JS ─────────
    // JS:   const x = 5   (مينفعش تعمل reassign)   /  let x = 5 (ينفع)
    // Rust: let x = 5     (immutable by default!)   /  let mut x = 5
    // الفلسفة: الـ default هو الأمان. لو عايز تغيّر لازم "تعلن" ده صراحةً.
    let x = 5;
    // x = 6; // ❌ error: cannot assign twice to immutable variable
    let mut counter = 0;
    counter += 1;
    println!("x={x}, counter={counter}");

    // ملحوظة مهمة: const في JS بيمنع الـ reassign بس، لكن ممكن تعدّل جوه الـ object:
    //   const arr = [1]; arr.push(2) // شغال عادي في JS
    // في Rust، `let v = vec![1]` مينفعش تعمل v.push(2) خالص — الـ immutability عميقة.
    let mut v = vec![1];
    v.push(2);
    println!("v = {v:?}");

    // ───────── Shadowing ─────────
    // ينفع تعرّف متغير بنفس الاسم تاني (وحتى بنوع مختلف). مفيد جداً في parsing:
    let port = "8080"; // &str
    let port: u16 = port.parse().unwrap(); // دلوقتي u16 — مش محتاج portStr و portNum
    println!("port = {port}");

    // ───────── الأرقام ─────────
    // JS عندها number واحد (float 64) + BigInt.
    // Rust بتخليك تختار الحجم بالظبط — لأنها language للـ systems والأداء.
    let a: i32 = -42; // signed 32-bit (الـ default للأرقام الصحيحة)
    let b: u8 = 255; // unsigned 8-bit: من 0 لـ 255
    let c: u64 = 10_000_000; // الـ _ للقراية بس
    let d: f64 = 2.5; // float (الـ default للكسور)
    let e = 7usize; // usize = حجم الـ pointer؛ بيُستخدم للـ indexes والأطوال
    println!("{a} {b} {c} {d} {e}");

    // مفيش implicit conversion خالص! في JS:  "5" * 2 === 10  😅
    // let bad = a + d; // ❌ مينفعش تجمع i32 مع f64
    let ok = a as f64 + d; // لازم تحوّل صراحةً
    println!("ok = {ok}");

    // الـ overflow: في debug بيعمل panic، وعندك methods صريحة:
    println!("checked: {:?}", b.checked_add(1)); // None بدل ما يعدي
    println!("saturating: {}", b.saturating_add(10)); // يقف عند 255
    println!("wrapping: {}", b.wrapping_add(1)); // يلف لـ 0

    // ───────── bool و char ─────────
    let active: bool = true;
    let letter: char = 'ع'; // char = Unicode scalar (4 bytes)، بـ single quotes
    println!("{active} {letter}");

    // مفيش truthy/falsy! لازم الشرط يكون bool:
    let n = 0;
    // if n { }        // ❌ JS كانت هتعتبرها false، Rust بترفض
    if n == 0 {
        println!("n صفر")
    }

    // ───────── String vs &str (أهم حاجة هنا) ─────────
    // &str   = "view" على نص موجود في مكان ما (read-only، مش بتملكه) — زي slice
    // String = نص إنت بتملكه على الـ heap، ينفع يكبر ويتعدّل
    let literal: &str = "hello"; // مخزّن جوه الـ binary نفسه
    let mut owned: String = String::from("hello"); // أو "hello".to_string()
    owned.push_str(" world");
    let view: &str = &owned[0..5]; // &str بيشاور على جزء من الـ String
    println!("{literal} | {owned} | {view}");
    // القاعدة العملية: الـ function arguments خليها &str، والـ struct fields خليها String.

    // ───────── Tuples و Arrays ─────────
    let pair: (i32, &str) = (1, "one");
    let (num, word) = pair; // destructuring زي JS
    println!("{num} {word} {}", pair.0);

    let arr: [i32; 3] = [1, 2, 3]; // حجم ثابت معروف وقت الـ compile (على الـ stack)
    let vec: Vec<i32> = vec![1, 2, 3]; // ده اللي يشبه Array في JS (بيكبر)
    println!("{arr:?} {vec:?} len={}", vec.len());
    // arr[10] → panic بدل undefined. ولو عايز نسخة آمنة:
    println!("vec.get(10) = {:?}", vec.get(10)); // None

    // ───────── const و static ─────────
    // const في Rust = قيمة معروفة وقت الـ compile (زي #define)، لازم تكتب النوع
    const MAX_RETRIES: u32 = 3;
    println!("MAX_RETRIES = {MAX_RETRIES}");

    // ───────── كل حاجة expression ─────────
    // مفيش ternary `? :` لأن if نفسها بترجع قيمة:
    let label = if counter > 0 { "positive" } else { "zero" };
    // حتى الـ block بيرجع قيمة:
    let y = {
        let t = 10;
        t * 2 // من غير ;
    };
    println!("{label} {y}");

    // الـ loops
    for i in 0..3 {
        print!("{i} ")
    } // 0..3 = [0,1,2]   ,  0..=3 = [0,1,2,3]
    println!();
    let mut tries = 0;
    let result = loop {
        // loop بترجع قيمة من break
        tries += 1;
        if tries == 3 {
            break tries * 10;
        }
    };
    println!("result = {result}");
}
