//! نفس مولّدات البيانات في نسخة نود، بنفس الأحجام والحقول.

use serde_json::{json, Value};

fn oid(seed: usize) -> String {
    format!("{:0>24x}", seed)
}

pub fn make_product(index: usize) -> Value {
    json!({
        "_id": oid(index + 1000),
        "handle": format!("product-{index}"),
        "title": format!("منتج تجريبي رقم {index}"),
        "description": "وصف المنتج بالتفصيل، فيه كلام كفاية عشان يشبه المحتوى الحقيقي اللي بيتخزّن في الداتابيز ويتبعت للقالب.",
        "status": "active",
        "onlineStoreOnly": true,
        "price": 100 + index,
        "compareAtPrice": 150 + index,
        "currency": "EGP",
        "images": (0..4).map(|i| json!({
            "_id": oid(index * 10 + i),
            "url": format!("https://cdn.example.com/media/store/product-{index}-{i}.webp"),
            "alt": format!("صورة {i} للمنتج {index}"),
            "width": 1200,
            "height": 1200
        })).collect::<Vec<_>>(),
        "variants": (0..3).map(|i| json!({
            "_id": oid(index * 100 + i),
            "title": format!("مقاس {i}"),
            "sku": format!("SKU-{index}-{i}"),
            "price": 100 + index + i,
            "inventory": { "quantity": 25 - i, "policy": "deny" },
            "options": [{ "name": "المقاس", "value": format!("{}", 38 + i) }]
        })).collect::<Vec<_>>(),
        "options": [{ "name": "المقاس", "values": ["38", "39", "40"] }],
        "collections": [oid(index + 5000)],
        "seo": { "title": format!("منتج {index}"), "description": "وصف السيو" },
        "createdAt": "2026-01-01T00:00:00.000Z"
    })
}

pub fn make_product_list(count: usize) -> Value {
    Value::Array((0..count).map(make_product).collect())
}

const SIMPLE_TYPES: [&str; 6] = ["text", "color", "richtext", "url", "number", "checkbox"];

/// ملامح الويدجت — نفس التقسيم في نسخة نود.
pub fn profile_fields(profile: &str) -> (usize, Vec<&'static str>) {
    match profile {
        "productGrid" => (6, vec!["products"]),
        "collectionGrid" => (6, vec!["collection_products"]),
        "navigation" => (5, vec!["menu"]),
        "featured" => (6, vec!["product", "media"]),
        _ => (8, vec![]),
    }
}

pub fn make_widget(index: usize, profile: &str) -> Value {
    let (simple, entity_types) = profile_fields(profile);
    let mut settings = serde_json::Map::new();
    let mut data = serde_json::Map::new();
    let mut field = 0usize;

    for i in 0..simple {
        settings.insert(
            format!("field_{field}"),
            json!({ "type": SIMPLE_TYPES[i % SIMPLE_TYPES.len()], "default": Value::Null }),
        );
        data.insert(
            format!("field_{field}"),
            json!(format!("قيمة نصية للحقل رقم {field} في القسم {index}")),
        );
        field += 1;
    }

    for ty in entity_types {
        settings.insert(format!("field_{field}"), json!({ "type": ty, "default": Value::Null }));
        data.insert(
            format!("field_{field}"),
            Value::Array((0..6).map(|k| json!(oid(k + 200))).collect()),
        );
        field += 1;
    }

    json!({
        "_id": oid(index + 100),
        "name": format!("widget-{index}"),
        "widgetKey": format!("section-{index}"),
        "profile": profile,
        "settings": Value::Object(settings),
        "data": Value::Object(data)
    })
}

pub fn make_block(index: usize) -> Value {
    let mut settings = serde_json::Map::new();
    let mut data = serde_json::Map::new();

    for i in 0..4 {
        settings.insert(
            format!("field_{i}"),
            json!({ "type": SIMPLE_TYPES[i % SIMPLE_TYPES.len()], "default": Value::Null }),
        );
        data.insert(format!("field_{i}"), json!(format!("نص البلوك {index} حقل {i}")));
    }

    json!({
        "_id": oid(index + 900),
        "name": format!("block-{index}"),
        "widgetKey": format!("block-{index}"),
        "settings": Value::Object(settings),
        "data": Value::Object(data)
    })
}
