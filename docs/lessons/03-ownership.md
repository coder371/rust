# 03 — Ownership و Borrowing ⭐ (أهم درس)

📄 الكود: [`examples/03_ownership.rs`](../examples/03_ownership.rs)

## المشكلة اللي Rust بتحلها

كل لغة لازم تجاوب على سؤال: **إمتى الـ memory دي تتمسح؟**

| اللغة | الإجابة | التمن |
|---|---|---|
| C / C++ | إنت بتمسحها يدوي (`free`) | use-after-free، double-free، memory leaks، ثغرات أمنية |
| JS / Java / Go | الـ GC بيلف ويمسح | pauses، memory أكتر، مفيش تحكم |
| **Rust** | الـ compiler بيحسبها وقت الـ compile | لازم تتعلم القواعد دي |

## القواعد الـ 3

1. **كل قيمة ليها owner واحد بس.**
2. **لما الـ owner يخرج من الـ scope → القيمة تتمسح** (فوراً، مش "بعدين لما الـ GC يفتكر").
3. **ينفع تستلف:** إما **أي عدد** من `&T` (قراية)، **أو** `&mut T` **واحد بس** (كتابة). مش الاتنين مع بعض.

## Move: أول صدمة

```js
// JS
const a = { name: "x" };
const b = a;          // الاتنين بيشاوروا على نفس الـ object
b.name = "y";
console.log(a.name);  // "y" 😱 — shared mutable state
```

```rust
// Rust
let a = String::from("x");
let b = a;            // الـ ownership "اتنقل". a خلاص مبقتش صالحة.
println!("{a}");      // ❌ compile error: value borrowed after move
```

Rust مش بتسمح بـ اتنين "مالكين" لنفس الحاجة. لو عايز نسخة → `.clone()` (صريح، وإنت عارف إن ليه تمن).

## Borrowing: الحل اليومي

```rust
fn print_len(s: &String) { println!("{}", s.len()); }   // استلاف للقراية
fn add_suffix(s: &mut String) { s.push_str("!"); }     // استلاف للكتابة

let mut name = String::from("rust");
print_len(&name);        // name لسه بتاعتي
add_suffix(&mut name);   // عدّلت فيها
```

**فكّر فيها كده:** `&` = "بصّ عليها ورجّعهالي"، `&mut` = "عدّل فيها ورجّعهالي"، من غير `&` = "خدها خلاص".

## ليه قاعدة "&mut واحد بس"؟

دي بتمنع كلاس كامل من الـ bugs:

```js
// JS — bug كلاسيك
const items = [1, 2, 3];
for (const x of items) {
  if (x === 2) items.push(4);  // بتعدّل وإنت بتلف → سلوك غريب
}
```

في Rust الكود ده **مش هيعمل compile**. ونفس القاعدة دي هي اللي بتمنع **data races** بين الـ threads (الدرس 10) — مجاناً.

## RAII: مفيش `finally { close() }`

```js
const conn = await pool.connect();
try { ... } finally { conn.release(); }   // لو نسيت → connection leak
```

في Rust أي حاجة ليها `Drop` بتتقفل لوحدها لما تخرج من الـ scope: الملفات، الـ locks، الـ DB connections، الـ transactions (لو معملتش commit → rollback تلقائي، هتشوفه في الدرس 13).

## الأخطاء المشهورة وحلولها

| الـ error | المعنى | الحل السريع |
|---|---|---|
| `value borrowed after move` | استخدمت حاجة بعد ما اديتها لحد | ابعت `&x` بدل `x`، أو `.clone()` |
| `cannot borrow as mutable because it is also borrowed as immutable` | فيه `&` لسه عايش وإنت عايز `&mut` | خلّص استخدام الـ `&` الأول، أو اعمل clone للقيمة اللي محتاجها |
| `cannot borrow as mutable, as it is not declared as mutable` | نسيت `mut` | `let mut x` |
| `borrowed value does not live long enough` | reference بيشاور على حاجة هتتمسح | رجّع owned value (`String`) بدل `&str` |

## جرّب بنفسك

1. شيل الكومنتات من الأسطر اللي عليها ❌ في الملف واحدة واحدة، واقرا كل error.
2. غيّر `takes_ownership(s)` لـ `takes_ownership(s.clone())` وشوف الفرق.
3. اكتب function `fn make_upper(s: &mut String)` تحوّل النص لـ uppercase في مكانه.
4. اكتب function `fn longest_word(text: &str) -> &str` ترجع أطول كلمة (من غير ما تعمل أي clone).
