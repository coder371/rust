# 05 — التعامل مع الأخطاء: Result و `?`

📄 الكود: [`examples/05_errors.rs`](../examples/05_errors.rs)

## الفلسفة

في Node، أي function ممكن تعمل `throw` — ومفيش أي حاجة في الـ signature بتقولك كده:

```ts
async function getUser(id: string): Promise<User>  // ممكن ترمي؟ مين يعرف 🤷
```

في Rust الـ error **جزء من النوع**:

```rust
async fn get_user(id: Uuid) -> Result<User, UserError>  // واضح: ممكن تفشل، وبالنوع ده
```

ومش هتقدر تستخدم الـ `User` من غير ما تتعامل مع احتمال الـ error. **الـ compiler بيجبرك.**

## `?` = الـ "throw" بتاعة Rust (بس ظاهر)

```ts
// TS
async function checkout() {
  const user = await getUser(id);        // لو فشلت → throw يطلع لفوق (مخفي)
  const cart = await getCart(user.id);
  return pay(cart);
}
```
```rust
// Rust
async fn checkout() -> Result<Receipt, AppError> {
    let user = get_user(id).await?;       // لو Err → return Err فوراً (ظاهر بالـ ?)
    let cart = get_cart(user.id).await?;
    pay(cart).await
}
```

نفس الشكل تقريباً، بس كل نقطة ممكن تفشل **معلّم عليها** بـ `?`.

## thiserror مقابل anyhow

| | `thiserror` | `anyhow` |
|---|---|---|
| لمين | modules / libraries | applications / main / scripts |
| شكل الـ error | enum بأنواع محددة | أي error (`Box<dyn Error>` + context) |
| اللي بيناديك يقدر يعمل match؟ | ✅ | ❌ (تقريباً) |
| مقابل في Node | `class NotFoundError extends Error` | `throw new Error("...")` + `cause` |

**قاعدة عملية في مشروع زي test-proj:**
- كل module (orders, inventory) عنده `enum OrdersError` بـ thiserror.
- الـ API layer بيعمل `match` عليه ويحوّله لـ HTTP status.
- الـ `main` والـ bin scripts بيستخدموا `anyhow::Result`.

## panic مقابل Result

| | `panic!` | `Result` |
|---|---|---|
| إمتى | bug في الكود، invariant اتكسر | حاجة متوقعة ممكن تحصل |
| مثال | index برّه الحدود، `unwrap` على None | ملف مش موجود، input غلط، DB وقعت |
| مقابل Node | `process.exit(1)` / crash | error بتتعامل معاه |

`unwrap()` في production code = **"أنا متأكد 100% ده مش هيفشل"**. لو مش متأكد، استخدم `?`. ولو متأكد، استخدم `expect("السبب")` عشان لو طلعت غلطان تعرف ليه.

## جرّب بنفسك

1. اكتب `fn parse_kv(line: &str) -> Result<(String, i32), ParseError>` بتحوّل `"age=30"`. الـ errors: `MissingEquals`, `EmptyKey`, `InvalidNumber(ParseIntError)` — باستخدام thiserror و `#[from]`.
2. اكتب `fn load(path) -> anyhow::Result<Vec<(String, i32)>>` تقرا ملف وتعمل parse لكل سطر، مع `.with_context(|| format!("line {n}"))`.
3. في `main` اطبع الـ error بـ `{e:#}` و `{e:?}` وقارن.
