//! نفس مراحل نسخة نود بنفس الترتيب، وبنفس عدد عمليات المونجو والريدس.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::Duration,
};

use minijinja::{Value as JValue, context};
use serde_json::{json, Map, Value};

use crate::{config::Config, data, templates};

pub static MONGO_OPS: AtomicU64 = AtomicU64::new(0);
pub static REDIS_OPS: AtomicU64 = AtomicU64::new(0);

async fn mongo(config: &Config) {
    MONGO_OPS.fetch_add(1, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_micros(config.mongo_us)).await;
}

async fn redis(config: &Config) {
    REDIS_OPS.fetch_add(1, Ordering::Relaxed);
    tokio::time::sleep(Duration::from_micros(config.redis_us)).await;
}

/// الكاش بيخزّن القيمة بالشكل اللي القالب بيستهلكه.
///
/// تخزين `serde_json::Value` كان بيخلّي كل إصابة كاش تعمل نسخة عميقة
/// للشجرة كلها + تحويل تاني للقالب. نود بيرجّع نفس المرجع، فده
/// المقابل العادل.
fn page_cache() -> &'static Mutex<HashMap<String, JValue>> {
    static CACHE: OnceLock<Mutex<HashMap<String, JValue>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// الأقسام: هيدر، بانرات، ٣ شبكات منتجات، فوتر — نفس نسخة نود
const SECTIONS: [&str; 10] = [
    "navigation", "static", "productGrid", "static", "collectionGrid",
    "featured", "productGrid", "static", "static", "navigation",
];

// ---------------------------------------------------------------
//  معالجة الحقول — FieldProcessorService
// ---------------------------------------------------------------

async fn process_field(config: &Config, ty: &str) -> Value {
    match ty {
        "products" | "collections" | "collection_products" => {
            mongo(config).await;
            let products = data::make_product_list(config.products_per_list);
            // applyTranslationService.applyMany — مرور على كل عنصر
            Value::Array(
                products
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| {
                        let mut clone = p.clone();
                        if let Some(title) = clone.get("title").and_then(|t| t.as_str()) {
                            let t = title.to_string();
                            clone["title"] = json!(t);
                        }
                        clone
                    })
                    .collect(),
            )
        }
        "product" | "article" | "blog" => {
            mongo(config).await;
            data::make_product(0)
        }
        "menu" => {
            mongo(config).await;
            Value::Array(
                (0..8)
                    .map(|i| json!({ "title": format!("قسم {i}"), "url": format!("/c/{i}") }))
                    .collect(),
            )
        }
        "media" | "image" => json!({
            "url": "https://cdn.example.com/media/banner.webp",
            "alt": "بانر", "width": 1600, "height": 600
        }),
        _ => Value::Null,
    }
}

/// dynamic-widget-process.service.ts — بيلفّ على كل حقل
async fn process_widget_data(config: &Config, widget: &Value) -> Value {
    let settings = widget["settings"].as_object().cloned().unwrap_or_default();
    let data_obj = widget["data"].as_object().cloned().unwrap_or_default();

    if config.parallel_fields {
        let futures_iter = settings.iter().map(|(key, setting)| {
            let ty = setting["type"].as_str().unwrap_or("text").to_string();
            let fallback = data_obj.get(key).cloned().unwrap_or(Value::Null);
            let key = key.clone();
            async move {
                let processed = process_field(config, &ty).await;
                (key, if processed.is_null() { fallback } else { processed })
            }
        });
        let pairs = futures::future::join_all(futures_iter).await;
        return Value::Object(pairs.into_iter().collect::<Map<_, _>>());
    }

    // asis: بالتتابع، كل حقل بيستنى اللي قبله
    let mut out = Map::new();
    for (key, setting) in settings.iter() {
        let ty = setting["type"].as_str().unwrap_or("text");
        let processed = process_field(config, ty).await;
        let value = if processed.is_null() {
            data_obj.get(key).cloned().unwrap_or(Value::Null)
        } else {
            processed
        };
        out.insert(key.clone(), value);
    }
    Value::Object(out)
}

// ---------------------------------------------------------------
//  تجميع الويدجتس — GetFinalWidgetsForRenderService
// ---------------------------------------------------------------

async fn get_final_widgets(config: &Config, cache_key: &str) -> JValue {
    // قراءة الكاش — متعلّقة في core-service الحالي
    if config.page_cache {
        let hit = page_cache().lock().unwrap().get(cache_key).cloned();
        if let Some(cached) = hit {
            redis(config).await;
            return cached;
        }
    }

    let main_sections = &SECTIONS[1..SECTIONS.len() - 1];

    // page.findOne(...).populate(...)
    mongo(config).await;
    let page_widgets: Vec<Value> = main_sections
        .iter()
        .enumerate()
        .map(|(i, profile)| data::make_widget(i + 1, profile))
        .collect();

    // templateModel.find({ key: { $in } }).populate(...)
    mongo(config).await;
    let header = data::make_widget(0, SECTIONS[0]);
    let footer = data::make_widget(99, SECTIONS[SECTIONS.len() - 1]);

    let templates_in_order: Vec<(&str, Vec<Value>)> = vec![
        ("header", vec![header]),
        ("main", page_widgets),
        ("footer", vec![footer]),
    ];

    let mut result = Vec::new();

    for (key, widgets) in templates_in_order {
        // blockModel.find({ _id: { $in } })
        mongo(config).await;
        let blocks: Vec<Value> = (0..widgets.len() * config.blocks_per_widget)
            .map(data::make_block)
            .collect();

        // metaobjectsResolver.resolve
        mongo(config).await;

        let mut final_widgets = Vec::new();
        let mut cursor = 0usize;

        for widget in &widgets {
            let widget_data = process_widget_data(config, widget).await;

            let mut widget_blocks = Vec::new();
            for _ in 0..config.blocks_per_widget {
                let block = &blocks[cursor];
                cursor += 1;
                widget_blocks.push(json!({
                    "blockKey": block["widgetKey"],
                    "name": block["name"],
                    "data": process_widget_data(config, block).await
                }));
            }

            let index: usize = widget["name"]
                .as_str()
                .and_then(|n| n.split('-').nth(1))
                .and_then(|n| n.parse().ok())
                .unwrap_or(0);

            // القالب بيقرا `products` صراحةً بدل ما يخمّن رقم الحقل —
            // الأقسام الساكنة مالهاش قائمة، فبتبقى فاضية.
            let list_key = widget["settings"].as_object().and_then(|settings| {
                settings.iter().find_map(|(key, setting)| {
                    matches!(
                        setting["type"].as_str(),
                        Some("products" | "collections" | "collection_products")
                    )
                    .then(|| key.clone())
                })
            });

            let products = list_key
                .and_then(|key| widget_data.get(&key).cloned())
                .unwrap_or_else(|| Value::Array(vec![]));

            let mut out = widget.clone();
            out["template"] = json!(format!("section-{}", index % 14));
            out["products"] = products;
            out["data"] = widget_data;
            out["blocks"] = Value::Array(widget_blocks);
            final_widgets.push(out);
        }

        result.push(json!({ "key": key, "widgets": final_widgets }));
    }

    let result = JValue::from_serialize(&Value::Array(result));

    // الكتابة شغالة في الوضعين — في asis محدش بيقرا اللي اتكتب
    page_cache()
        .lock()
        .unwrap()
        .insert(cache_key.to_string(), result.clone());
    redis(config).await;

    result
}

// ---------------------------------------------------------------
//  الطلب كامل
// ---------------------------------------------------------------

pub async fn render_storefront_page(config: &Config) -> Result<String, minijinja::Error> {
    // store-context.middleware.ts
    mongo(config).await; // solveApp
    let store = json!({
        "_id": "store-1",
        "name": "متجر تجريبي",
        "url": "https://example.com",
        "logo": { "url": "https://cdn.example.com/logo.webp", "alt": "شعار", "width": 400, "height": 120 }
    });
    mongo(config).await; // resolveCustomDomain
    redis(config).await; // كاش الـ GeoIP
    redis(config).await; // كاش السوق
    let market = json!({ "_id": "market-eg", "code": "EG" });
    let currency = json!({ "code": "EGP", "rate": 1 });

    // init.middleware.ts — بحث وإنشاء للجهاز والجلسة
    for _ in 0..4 {
        mongo(config).await;
    }

    // theme.middleware.ts
    mongo(config).await;
    let app_theme = json!({
        "_id": "theme-1",
        "themeVersion": { "version": "1.0.0", "theme": "t-1" },
        "settings_data": { "colors": { "primary": "#111" }, "typography": { "base": 16 } }
    });
    redis(config).await;
    let translations = json!({ "add_to_cart": "أضف للسلة" });

    let templates_data = get_final_widgets(config, "store-1:index").await;

    // السياق بيتبني لقيم القالب مباشرة. لفّه في `json!` الأول كان
    // بيحوّل الشجرة الجاهزة ذهاباً وعودة بلا داعي.
    let ctx = context! {
        globals => context! { store => JValue::from_serialize(&store) },
        context => context! { theme => JValue::from_serialize(&app_theme) },
        localization => context! {
            market => JValue::from_serialize(&market),
            currency => JValue::from_serialize(&currency),
            language => "ar",
            availableLanguages => vec!["ar", "en"],
        },
        page => context! {
            handle => "index",
            title => "الرئيسية",
            description => "صفحة المتجر الرئيسية",
            path => "/",
        },
        translations => JValue::from_serialize(&translations),
        templates => templates_data,
    };

    templates::render(config, "layouts/layout.njk", ctx)
}
