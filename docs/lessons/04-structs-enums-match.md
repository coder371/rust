# 04 — Structs و Enums و Pattern Matching

📄 الكود: [`examples/04_structs_enums.rs`](../examples/04_structs_enums.rs)

## Struct + impl بدل class

```ts
// TS
class User {
  constructor(public id: number, public name: string) {}
  displayName() { return `#${this.id} ${this.name}`; }
}
```
```rust
// Rust
struct User { id: u64, name: String }        // الداتا

impl User {                                    // السلوك
    fn new(id: u64, name: &str) -> Self { Self { id, name: name.into() } }
    fn display_name(&self) -> String { format!("#{} {}", self.id, self.name) }
}
```

**مفيش inheritance.** بدلها:
- **Composition**: struct جواه struct تاني.
- **Traits**: سلوك مشترك (الدرس 06).

## الـ `self` التلاتة

| الـ signature | المعنى | مثال |
|---|---|---|
| `&self` | بيقرا بس | getters، حسابات |
| `&mut self` | بيعدّل | setters، `push` |
| `self` | بيستهلك الـ object | builders، `into_*` conversions |

## Enums = Discriminated Unions (بس أقوى)

```ts
// TS
type Shape =
  | { kind: "circle"; r: number }
  | { kind: "rect"; w: number; h: number };

function area(s: Shape) {
  switch (s.kind) {
    case "circle": return Math.PI * s.r ** 2;
    case "rect": return s.w * s.h;
    // لو زودت "triangle" ونسيت تعالجه؟ TS مش هتقول حاجة إلا لو عملت trick بـ never
  }
}
```
```rust
// Rust
enum Shape { Circle { r: f64 }, Rect { w: f64, h: f64 } }

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => std::f64::consts::PI * r * r,
        Shape::Rect { w, h } => w * h,
        // لو زودت Triangle → compile error: non-exhaustive patterns ✅
    }
}
```

الـ enums في Rust هي **أهم أداة تصميم**. استخدمها لأي حاجة ليها "حالات": status الـ order، نوع الـ event، نتيجة عملية.

**مبدأ: "Make illegal states unrepresentable"** — صمّم الأنواع بحيث الحالة الغلط متتكتبش أصلاً. مثلاً `Shipped { tracking: String }` — مستحيل يبقى عندك order "shipped" من غير tracking number.

## Option بدل null

```rust
enum Option<T> { Some(T), None }   // ده كل الموضوع — enum عادي
```

| JS | Rust |
|---|---|
| `user?.name` | `user.map(\|u\| u.name)` أو `user.as_ref().map(...)` |
| `x ?? "default"` | `x.unwrap_or("default")` |
| `if (x) { use(x) }` | `if let Some(x) = x { use(x) }` |
| `x!` (TS non-null) | `x.unwrap()` أو `x.expect("reason")` |

## جرّب بنفسك

1. زوّد variant جديد `Refunded { amount: f64 }` لـ `OrderStatus` واعمل build — شوف الـ compiler بيقولك فين نسيت.
2. اعمل enum `Command { Create(String), Delete(u64), List }` وfunction `parse(input: &str) -> Option<Command>` بتحوّل `"create foo"` / `"delete 5"` / `"list"`.
3. اعمل struct `Cart { items: Vec<(String, f64)> }` بـ methods: `add`, `total`, `is_empty`.
