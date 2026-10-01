# 14 — MongoDB

📄 الكود: [`examples/14_mongodb.rs`](../examples/14_mongodb.rs)

```bash
docker compose up -d mongo
cargo run --example 14_mongodb
```

## مفيش mongoose... وده مقصود

| mongoose | Rust |
|---|---|
| `new Schema({...})` | `#[derive(Serialize, Deserialize)] struct` |
| `mongoose.model('Product', schema)` | `db.collection::<Product>("products")` |
| validation في الـ schema | الأنواع نفسها + الـ deserialize (لو document بايظ → Err) |
| `required: true` | field عادي (مش `Option`) |
| optional field | `Option<T>` + `#[serde(default)]` |
| `pre('save')` hooks | functions عادية في الـ repository |
| `populate()` | `$lookup` في aggregation، أو query تانية |
| `.lean()` | دايماً lean — مفيش "mongoose documents" بـ magic |

الـ driver الرسمي `mongodb` (من MongoDB Inc نفسها) قريب جداً من الـ Node driver الرسمي، و`doc!{}` macro بيخلي الـ queries شكلها زي JS بالظبط.

## المقارنة

```js
// Node driver
const col = db.collection('products');
await col.insertOne({ sku: 'A', price: 10 });
const p = await col.findOne({ sku: 'A' });            // any
const list = await col.find({ price: { $gte: 5 } }).sort({ price: -1 }).limit(10).toArray();
await col.updateOne({ sku: 'A' }, { $inc: { stock: -1 } });
```
```rust
// Rust
let col: Collection<Product> = db.collection("products");
col.insert_one(&product).await?;
let p: Option<Product> = col.find_one(doc! { "sku": "A" }).await?;   // typed!
let list: Vec<Product> = col.find(doc! { "price": { "$gte": 5 } })
    .sort(doc! { "price": -1 }).limit(10).await?
    .try_collect().await?;
col.update_one(doc! { "sku": "A" }, doc! { "$inc": { "stock": -1 } }).await?;
```

## `_id` و ObjectId

```rust
#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
id: Option<ObjectId>,
```
- `None` وقت الـ insert → Mongo يولّده.
- لو عايز IDs بتاعتك (UUID)، خليه `id: Uuid` مع `rename = "_id"`.

## Postgres ولا Mongo؟

مش سؤال Rust، بس مهم: Rust type system بيدّيك **"schema" في الكود** حتى مع Mongo. عشان كده الـ schemaless مش بيفرق كتير في Rust زي ما بيفرق في JS. اختار حسب شكل الداتا والـ transactions اللي محتاجها.

## جرّب بنفسك

1. اعمل `ProductRepo { col: Collection<Product> }` بـ methods typed.
2. اعمل collection `orders` فيها `items: Vec<OrderItem>` (embedded) واعمل aggregation يحسب أكتر 3 منتجات مبيعاً.
3. اعمل text index على `name` وابحث بـ `$text`.
