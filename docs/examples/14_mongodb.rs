//! الدرس 14 — MongoDB
//! قبل ما تشغّل:  docker compose up -d mongo
//! شغّل:          cargo run --example 14_mongodb
//!
//! ═══ الفلسفة ═══
//! في Node غالباً بتستخدم mongoose: Schema + Model + validation + hooks + populate.
//! في Rust مفيش mongoose — والسبب إنك مش محتاجه:
//!   - الـ Schema    = struct + serde  (الـ type system نفسه هو الـ schema)
//!   - الـ Model     = Collection<T>   (collection typed: insert/find بيرجعوا T مش any)
//!   - الـ validation = deserialize: لو document في الـ DB شكله غلط → Err وقت القراية، مش undefined بعدين
//!   - الـ hooks     = functions عادية في الـ repository بتاعك
//!
//! الـ driver الرسمي (mongodb crate) من MongoDB نفسها، async، والـ API قريب جداً من Node driver.

use anyhow::Context;
use futures::TryStreamExt;
use mongodb::bson::{DateTime as BsonDateTime, Document, doc, oid::ObjectId};
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    // _id في Mongo ↔ id في Rust. skip_serializing_if عشان Mongo يولّده لو None
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    sku: String,
    name: String,
    price: f64,
    stock: i32,
    tags: Vec<String>,
    // nested document. skip_serializing_if مهمة هنا: من غيرها None بيتخزن null،
    // وبعدين {"$set": {"attributes.color": ...}} يفشل: Cannot create field 'color' in element {attributes: null}
    #[serde(default, skip_serializing_if = "Option::is_none")]
    attributes: Option<Attributes>,
    created_at: BsonDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Attributes {
    color: Option<String>,
    size: Option<String>,
}

// projection/aggregation results ليها struct خاصة بيها
#[derive(Debug, Deserialize)]
struct TagStats {
    #[serde(rename = "_id")]
    tag: String,
    count: i32,
    avg_price: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = std::env::var("MONGO_URL").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    // ═══ 1) Connection ═══
    // Client = connection pool جواه (زي MongoClient في Node). اعمله مرة واحدة واعمله clone.
    let mut opts = ClientOptions::parse(&url).await?;
    opts.app_name = Some("rust-learn".into());
    opts.server_selection_timeout = Some(std::time::Duration::from_secs(3));
    let client = Client::with_options(opts)?;
    client
        .database("admin")
        .run_command(doc! { "ping": 1 })
        .await
        .context("مش قادر أتصل بـ Mongo — شغّلت docker compose up -d mongo؟")?;
    println!("✅ connected");

    let db = client.database("learn");
    // Collection<Product>: كل العمليات هتبقى typed
    let products: Collection<Product> = db.collection("products");
    products.drop().await?; // نبدأ على نضيف كل مرة

    // ═══ 2) Indexes ═══ (زي schema.index({ sku: 1 }, { unique: true }))
    products
        .create_index(
            IndexModel::builder()
                .keys(doc! { "sku": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    // ═══ 3) Insert ═══
    let now = BsonDateTime::now();
    let tshirt = Product {
        id: None,
        sku: "TSHIRT-RED-M".into(),
        name: "تيشيرت أحمر".into(),
        price: 250.0,
        stock: 10,
        tags: vec!["clothes".into(), "summer".into()],
        attributes: Some(Attributes {
            color: Some("red".into()),
            size: Some("M".into()),
        }),
        created_at: now,
    };
    let res = products.insert_one(&tshirt).await?;
    println!("inserted id = {}", res.inserted_id);

    let many = vec![
        Product {
            sku: "JEANS-32".into(),
            name: "جينز".into(),
            price: 600.0,
            stock: 3,
            tags: vec!["clothes".into()],
            attributes: None,
            ..tshirt.clone()
        },
        Product {
            sku: "CAP-01".into(),
            name: "كاب".into(),
            price: 120.0,
            stock: 0,
            tags: vec!["summer".into(), "accessories".into()],
            attributes: None,
            ..tshirt.clone()
        },
        Product {
            sku: "WATCH-9".into(),
            name: "ساعة".into(),
            price: 1500.0,
            stock: 5,
            tags: vec!["accessories".into()],
            attributes: None,
            ..tshirt.clone()
        },
    ];
    let r = products.insert_many(&many).await?;
    println!("inserted many = {}", r.inserted_ids.len());

    // duplicate key error (code 11000) — زي err.code === 11000 في Node
    if let Err(e) = products.insert_one(&tshirt).await {
        let dup = matches!(
            *e.kind,
            mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(ref we)) if we.code == 11000
        );
        println!("⚠️ duplicate sku? {dup}");
    }

    // ═══ 4) Find ═══ doc! macro = نفس شكل الـ query في JS
    let one = products.find_one(doc! { "sku": "JEANS-32" }).await?;
    println!("find_one: {:?}", one.map(|p| p.name));

    // find + options (sort / limit / projection) — الـ builder syntax بدل object options
    let mut cursor = products
        .find(doc! { "price": { "$gte": 200 }, "stock": { "$gt": 0 } })
        .sort(doc! { "price": -1 })
        .limit(10)
        .await?;
    println!("in stock & price >= 200:");
    // Cursor = async stream. try_next بيرجع Result<Option<T>>
    while let Some(p) = cursor.try_next().await? {
        println!("  {} — {} EGP (stock {})", p.name, p.price, p.stock);
    }

    // أو collect كله مرة واحدة (زي .toArray())
    let summer: Vec<Product> = products
        .find(doc! { "tags": "summer" })
        .await?
        .try_collect()
        .await?;
    println!(
        "summer: {:?}",
        summer.iter().map(|p| &p.sku).collect::<Vec<_>>()
    );

    // ═══ 5) Update ═══
    // $inc atomic — المهم في الـ stock: متعملش read ثم write (race condition)
    // الشرط stock >= 2 جوه الـ filter = "خصم بس لو فيه كفاية" في عملية واحدة atomic
    let upd = products
        .update_one(
            doc! { "sku": "JEANS-32", "stock": { "$gte": 2 } },
            doc! { "$inc": { "stock": -2 }, "$push": { "tags": "sale" } },
        )
        .await?;
    println!(
        "reserve 2 jeans: matched={} modified={}",
        upd.matched_count, upd.modified_count
    );
    let upd = products
        .update_one(
            doc! { "sku": "JEANS-32", "stock": { "$gte": 2 } },
            doc! { "$inc": { "stock": -2 } },
        )
        .await?;
    println!("reserve 2 more: matched={} (مفيش كفاية)", upd.matched_count);

    // find_one_and_update بيرجع الـ document بعد التعديل
    let updated = products
        .find_one_and_update(
            doc! { "sku": "CAP-01" },
            doc! { "$set": { "stock": 20, "attributes.color": "black" } },
        )
        .return_document(mongodb::options::ReturnDocument::After)
        .await?;
    println!(
        "cap after update: {:?}",
        updated.map(|p| (p.stock, p.attributes))
    );

    // upsert
    products
        .update_one(
            doc! { "sku": "SOCKS-1" },
            doc! { "$setOnInsert": { "name": "شراب", "price": 50.0, "stock": 100, "tags": [], "created_at": BsonDateTime::now() } },
        )
        .upsert(true)
        .await?;

    // ═══ 6) Aggregation ═══
    // الـ pipeline بيرجع Document (مش typed) — بنحوّله بـ bson::from_document لـ struct
    let pipeline = vec![
        doc! { "$unwind": "$tags" },
        doc! { "$group": { "_id": "$tags", "count": { "$sum": 1 }, "avg_price": { "$avg": "$price" } } },
        doc! { "$sort": { "count": -1, "_id": 1 } },
    ];
    let mut agg = products.aggregate(pipeline).await?;
    println!("tag stats:");
    while let Some(d) = agg.try_next().await? {
        let s: TagStats = mongodb::bson::from_document(d)?;
        println!("  {:<12} count={} avg={:.1}", s.tag, s.count, s.avg_price);
    }

    // ═══ 7) Raw documents (زي any) ═══
    let raw: Collection<Document> = db.collection("products");
    if let Some(d) = raw
        .find_one(doc! {})
        .projection(doc! { "name": 1, "_id": 0 })
        .await?
    {
        println!("raw projection: {d}");
    }

    // ═══ 8) Delete ═══
    let del = products.delete_many(doc! { "stock": 0 }).await?;
    println!("deleted out-of-stock: {}", del.deleted_count);
    println!("total now: {}", products.count_documents(doc! {}).await?);

    // ═══ 9) Transactions ═══ محتاجة replica set (الـ docker image العادي standalone)
    // let mut session = client.start_session().await?;
    // session.start_transaction().await?;
    // products.update_one(...).session(&mut session).await?;
    // session.commit_transaction().await?;
    Ok(())
}
