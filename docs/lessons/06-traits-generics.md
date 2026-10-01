# 06 — Traits و Generics

📄 الكود: [`examples/06_traits.rs`](../examples/06_traits.rs)

## Trait = Interface (مع فروق مهمة)

```ts
interface Notifier { send(to: string, msg: string): Promise<void>; }
class Email implements Notifier { ... }
// أي object فيه send بنفس الشكل هيتقبل (structural / duck typing)
```
```rust
trait Notifier { fn send(&self, to: &str, msg: &str) -> Result<(), String>; }
impl Notifier for Email { ... }   // لازم تعلن صراحةً (nominal)
```

الفروق:
1. **ينفع تعمل implement لـ trait على نوع مش بتاعك.** مثلاً `impl MyTrait for String`. (في JS ده monkey-patching خطير — هنا آمن وscoped.)
2. **Default methods** — زي abstract class بـ implementation جاهز.
3. **مفيش inheritance للـ data** — traits للسلوك بس.

## Static مقابل Dynamic Dispatch

```rust
fn notify(n: &impl Notifier)        // static:  نسخة من الـ function لكل نوع (الأسرع)
fn notify<N: Notifier>(n: &N)       // static:  نفس اللي فوق
fn notify(n: &dyn Notifier)         // dynamic: vtable وقت التشغيل (زي JS)
Vec<Box<dyn Notifier>>              // لما تحتاج أنواع مختلفة في نفس الـ list
```

**الـ default:** استخدم generics/`impl Trait`. استخدم `dyn` لما تحتاج تحط أنواع مختلفة في نفس المكان (plugins، strategies، list of handlers).

## Generics: مش بتتمسح!

في TS: `function f<T>(x: T)` → بعد الـ compile بتبقى `function f(x)`. الأنواع اختفت.

في Rust: `fn f<T>(x: T)` → الـ compiler بيولّد `f_i32`, `f_String`, ... لكل نوع بتستخدمه (**monomorphization**). النتيجة: نفس سرعة الكود المكتوب يدوي. ده معنى **zero-cost abstractions**.

## Traits مشهورة لازم تعرفها

| Trait | بيعمل إيه | مقابل JS |
|---|---|---|
| `Debug` | `{:?}` | `console.dir` / `util.inspect` |
| `Display` | `{}` و `.to_string()` | `toString()` |
| `Clone` | `.clone()` | `structuredClone` |
| `PartialEq` | `==` | مفيش (JS بتقارن references) |
| `Default` | `T::default()` | default values |
| `From` / `Into` | تحويل بين أنواع | constructors / factory functions |
| `Serialize` / `Deserialize` | JSON وغيره (serde) | `JSON.stringify` / `parse` |
| `Iterator` | `for x in ...` | `[Symbol.iterator]` |
| `Drop` | cleanup | مفيش (finalizers مش موثوقة) |
| `Send` / `Sync` | آمن بين threads | مفيش |

أغلبهم بيتعملوا أوتوماتيك بـ `#[derive(...)]`.

## جرّب بنفسك

1. اعمل trait `Storage { fn get(&self, k: &str) -> Option<String>; fn set(&mut self, k: &str, v: String); }` واعمل implementation بـ `HashMap`. (ده بالظبط الـ pattern اللي بيخليك تعمل mock للـ DB في الـ tests.)
2. اعمل `impl Display for User` واطبعه بـ `{}`.
3. اكتب `fn total<T: Into<f64> + Copy>(items: &[T]) -> f64`.
