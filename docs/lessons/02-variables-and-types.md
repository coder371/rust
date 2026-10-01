# 02 — المتغيرات والأنواع

📄 الكود: [`examples/02_variables.rs`](../examples/02_variables.rs)

## الفلسفة: immutable by default

| JS | Rust | المعنى |
|---|---|---|
| `const x = 5` | `let x = 5` | مينفعش تغيّرها |
| `let x = 5` | `let mut x = 5` | ينفع تغيّرها |
| `const arr = []; arr.push(1)` ✅ | `let v = vec![]; v.push(1)` ❌ | في Rust الـ immutability **عميقة** |

الفكرة: لو شفت `let mut`، يبقى فيه حاجة هتتغير — خلي بالك. لو شفت `let`، اطمن.

## الأرقام: لازم تختار الحجم

JS عندها `number` (float 64-bit) لكل حاجة. عشان كده `0.1 + 0.2 !== 0.3` و`2**53 + 1` بيبوظ.

Rust بتخليك تختار: `i8..i128`, `u8..u128`, `f32`, `f64`, `usize`.
- **الفلوس؟** استخدم `i64` بالقروش (زي ما هنعمل في الـ DB) أو crate زي `rust_decimal`.
- **IDs و counts؟** `u64` / `i64`.
- **indexes وأطوال؟** `usize`.

ومفيش **implicit conversion خالص**. `"5" * 2` في JS = `10`. في Rust مش هيعمل compile.

## مفيش truthy / falsy

```js
if (count) { ... }         // JS: 0 و "" و null و NaN كلهم false
```
```rust
if count > 0 { ... }       // Rust: الشرط لازم يكون bool بالظبط
if let Some(x) = maybe {}  // بدل if (maybe)
if !s.is_empty() {}        // بدل if (str)
```

## String مقابل &str

دي أكتر حاجة بتلخبط في الأول:

| | `String` | `&str` |
|---|---|---|
| مين بيملكه | إنت | حد تاني (إنت مستلفه) |
| مكانه | heap | أي حتة (binary، heap، stack) |
| ينفع يتعدل | أيوه (لو mut) | لأ |
| إمتى تستخدمه | struct fields، return values جديدة | function parameters |

```rust
fn greet(name: &str) -> String {      // استلف، ورجّع حاجة جديدة بتملكها
    format!("Hello {name}")
}
greet("literal");                     // &str
greet(&my_string);                    // &String → &str تلقائي
```

## كل حاجة expression

مفيش ternary لأن `if` نفسها بترجع قيمة. و`match` كمان. والـ block `{}` كمان. ده بيقلل المتغيرات المؤقتة و`let x; if (...) x = ... else x = ...`.

## جرّب بنفسك

1. اعمل `let x = 5;` وبعدين `x = 6;` — اقرا الـ error.
2. جرّب `let b: u8 = 256;` — الـ compiler هيمسكها.
3. اكتب function `fn fahrenheit(c: f64) -> f64` وجرّبها.
4. اكتب loop بيعدّ من 1 لـ 20 ويطبع FizzBuzz باستخدام `match (i % 3, i % 5)`.
