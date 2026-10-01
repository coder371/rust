//! الدرس 01 — أول برنامج + الفرق بين Cargo و npm
//! شغّل:  cargo run --example 01_hello
//!
//! في Node:  node index.js   → الكود بيتفسَّر ويتنفذ وقت التشغيل (JIT)
//! في Rust:  cargo run       → بيتعمل compile لـ binary حقيقي، وبعدين يتنفذ
//!           أي غلط في الأنواع بيطلع وقت الـ compile — مش في production الساعة 3 الفجر.

// `fn main` هي نقطة البداية. مفيش "top-level code" زي JS.
fn main() {
    // println! فيها `!` لأنها macro مش function (هتفهم ليه تحت)
    println!("أهلاً يا Rust 🦀");

    // الـ template string في JS:  `Hello ${name}`
    let name = "Ahmed";
    println!("Hello {name}"); // الأسهل: اسم المتغير جوه {}
    println!("Hello {}", name); // أو positional
    println!("debug: {:?}", vec![1, 2]); // {:?} = Debug، زي console.dir تقريباً
    println!("pretty: {:#?}", ("a", 1)); // {:#?} = Debug متنسق على كذا سطر

    // eprintln = console.error (بيكتب على stderr)
    eprintln!("ده رايح على stderr");

    // format! بترجع String بدل ما تطبع — زي template literal
    let msg = format!("{} + {} = {}", 2, 3, add(2, 3));
    println!("{msg}");

    // ليه println macro؟ لأن Rust مفيهاش functions بعدد arguments متغير (variadic).
    // الـ macro بيتفرد وقت الـ compile ويتأكد إن عدد الـ {} = عدد الـ arguments.
    // جرّب تشيل الكومنت من السطر ده وشوف الـ compiler هيقولك إيه:
    // println!("{} {}", 1);
}

// آخر expression من غير `;` هو قيمة الـ return — مفيش حاجة اسمها "undefined"
fn add(a: i32, b: i32) -> i32 {
    a + b // ← لو حطيت `;` هنا هتبقى statement وترجع () وهيطلعلك compile error
}
