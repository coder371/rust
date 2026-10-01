//! محرّك القوالب — MiniJinja، المقابل المباشر لـ Nunjucks.
//!
//! وضع `asis` بيقلّد `noCache: true`: بيبني بيئة جديدة ويقرا القوالب من
//! القرص ويترجمها في كل طلب. وضع `tuned` بيبنيها مرة واحدة.

use std::sync::OnceLock;

use minijinja::{AutoEscape, Environment, Value as JValue, path_loader};
use crate::config::Config;

fn theme_dir() -> String {
    std::env::var("THEME_DIR").unwrap_or_else(|_| "theme".to_string())
}

/// الفلاتر اللي القوالب بتناديها فعلاً — نفس اللي في نسخة نود.
fn add_filters(env: &mut Environment<'static>) {
    // لازم يشتغل مع الأرقام كمان — `width` و`height` أرقام مش نصوص،
    // و`as_str` لوحدها بترجّع فاضي فيهم.
    let attr_str = |value: &JValue, key: &str| -> String {
        match value.get_attr(key) {
            Ok(v) if v.is_undefined() || v.is_none() => String::new(),
            Ok(v) => v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string()),
            Err(_) => String::new(),
        }
    };

    env.add_filter("money", |value: JValue, currency: Option<String>| {
        let amount = f64::try_from(value.clone()).unwrap_or(0.0);
        format!("{:.2} {}", amount, currency.unwrap_or_else(|| "EGP".into()))
    });

    env.add_filter("image_url", move |image: JValue, width: Option<u32>| {
        let url = attr_str(&image, "url");
        if url.is_empty() {
            return String::new();
        }
        format!("{url}?w={}&fm=webp", width.unwrap_or(800))
    });

    env.add_filter("image_tag", move |image: JValue, width: Option<u32>| {
        let url = attr_str(&image, "url");
        if url.is_empty() {
            return String::new();
        }
        let alt = attr_str(&image, "alt");
        let w = attr_str(&image, "width");
        let h = attr_str(&image, "height");
        format!(
            "<img src=\"{url}?w={}\" alt=\"{alt}\" width=\"{w}\" height=\"{h}\" loading=\"lazy\">",
            width.unwrap_or(800)
        )
    });

    env.add_filter("truncate_words", |text: JValue, count: Option<usize>| {
        text.as_str()
            .unwrap_or_default()
            .split_whitespace()
            .take(count.unwrap_or(20))
            .collect::<Vec<_>>()
            .join(" ")
    });

    env.add_filter("handleize", |text: JValue| {
        text.as_str()
            .unwrap_or_default()
            .trim()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
    });

    env.add_filter("json_attr", |value: JValue| {
        serde_json::to_string(&value)
            .unwrap_or_default()
            .replace('"', "&quot;")
    });

    // فلاتر إضافية عشان عدد المسجَّلين يقرب من الحقيقي (٩٧ في core-service)
    for i in 0..90 {
        env.add_filter(
            Box::leak(format!("noop_{i}").into_boxed_str()) as &'static str,
            |value: JValue| value,
        );
    }
}

fn build_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(path_loader(theme_dir()));
    // MiniJinja بتقرر التهريب من امتداد الملف، و`.njk` مش في قائمتها.
    // Nunjucks عندها `autoescape: true` لكل القوالب، فبنطابقها.
    env.set_auto_escape_callback(|_| AutoEscape::Html);
    add_filters(&mut env);
    env
}

static CACHED: OnceLock<Environment<'static>> = OnceLock::new();

pub fn render(config: &Config, template: &str, ctx: JValue) -> Result<String, minijinja::Error> {
    if config.tpl_cache {
        // البيئة اتبنت مرة واحدة، والقوالب المترجمة محفوظة جوّاها
        let env = CACHED.get_or_init(build_env);
        return env.get_template(template)?.render(ctx);
    }

    // asis: بيئة جديدة كل طلب — قراءة من القرص وترجمة من الصفر،
    // زي `noCache: true` بالظبط
    let env = build_env();
    env.get_template(template)?.render(ctx)
}
