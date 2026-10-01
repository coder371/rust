# 12 — JSON و Serde

📄 الكود: [`examples/12_serde.rs`](../examples/12_serde.rs)

## الفلسفة: الـ parse هو الـ validation

```ts
// Node
const body = JSON.parse(raw);              // any 😬
const order = OrderSchema.parse(body);     // zod عشان تتأكد
```
```rust
// Rust
let order: CreateOrder = serde_json::from_str(raw)?;   // parse + validation في خطوة واحدة
```

لو field ناقص، أو نوعه غلط، أو رقم سالب في `u32`، أو uuid بايظ → `Err` برسالة واضحة فيها السطر والعمود.

**"Parse, don't validate"**: بدل ما تعمل validate لـ object وتفضل شايل `any`، حوّله لنوع مضمون. بعد كده الكود كله بيتعامل مع `CreateOrder` مش `any`.

## الـ attributes اللي هتستخدمها كل يوم

| Attribute | بيعمل إيه |
|---|---|
| `#[serde(rename_all = "camelCase")]` | `customer_id` ↔ `customerId` |
| `#[serde(rename = "x")]` | اسم مختلف لـ field واحد |
| `#[serde(default)]` | لو مش موجود → `Default::default()` |
| `#[serde(skip_serializing_if = "Option::is_none")]` | متكتبش الـ field لو None |
| `#[serde(skip)]` | تجاهله خالص |
| `#[serde(flatten)]` | زي `...spread` |
| `#[serde(tag = "type")]` | enums → `{ "type": "order_created", ... }` |
| `#[serde(deny_unknown_fields)]` | ارفض أي field زيادة (strict) |

## serde مش JSON بس

serde = framework. نفس الـ `#[derive(Serialize, Deserialize)]` بيشتغل مع JSON، BSON (Mongo)، YAML، TOML، MessagePack، CSV، query strings... عشان كده هتلاقيه في كل حتة في الدروس الجاية.

## جرّب بنفسك

1. زوّد `#[serde(deny_unknown_fields)]` على `CreateOrder` وابعت field زيادة.
2. اعمل enum `Currency { EGP, USD }` واستخدمه في `Item` — جرّب تبعت `"EUR"`.
3. اقرا ملف `config.json` فيه `{ "port": 8080, "db": { "url": "...", "pool": 10 } }` لـ struct متداخل.
