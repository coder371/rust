/**
 * نسخة طبق الأصل من NunjucksService في core-service.
 *
 * النقطة الحاسمة: الـ loader بيعمل `fs.readFileSync` وبيرجّع `noCache: true`،
 * والـ Environment نفسها `noCache: true`. يعني كل قالب — واللي بيتنادى من
 * جوّه بـ include كمان — بيتقري من القرص ويتترجم من أول وجديد في كل طلب.
 *
 * وضع `tuned` بيغيّر ده وبس: نفس القوالب، نفس المخرجات، كاش شغّال.
 */
import nunjucks from "nunjucks";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { config } from "./config.js";

const THEMES_REPO = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "theme",
);

class ThemeLoader extends nunjucks.Loader {
  constructor(noCache) {
    super();
    this.noCacheFlag = noCache;
  }

  getSource(name) {
    const fullPath = path.join(THEMES_REPO, name);
    const src = fs.readFileSync(fullPath, "utf-8");
    return { src, path: fullPath, noCache: this.noCacheFlag };
  }
}

const noCache = !config.njkCache;

export const env = new nunjucks.Environment(new ThemeLoader(noCache), {
  autoescape: true,
  noCache,
  watch: false,
});

// ---------------------------------------------------------------
//  الفلاتر — core-service بيسجّل ٩٧ فلتر و٢٣ إضافة.
//  اللي بيكلّف في الطلب هو النداء وقت الرندر، فالقوالب بتناديهم فعلاً.
// ---------------------------------------------------------------

env.addFilter("money", (value, currency = "EGP") => {
  const amount = Number(value ?? 0);
  return `${amount.toFixed(2)} ${currency}`;
});

env.addFilter("image_url", (image, width = 800) => {
  if (!image?.url) return "";
  return `${image.url}?w=${width}&fm=webp`;
});

env.addFilter("image_tag", (image, width = 800) => {
  if (!image?.url) return "";
  return `<img src="${image.url}?w=${width}" alt="${image.alt ?? ""}" width="${image.width}" height="${image.height}" loading="lazy">`;
});

env.addFilter("truncate_words", (text, count = 20) =>
  String(text ?? "").split(/\s+/).slice(0, count).join(" "),
);

env.addFilter("handleize", (text) =>
  String(text ?? "").trim().toLowerCase().replace(/\s+/g, "-"),
);

env.addFilter("t", (key, translations) => translations?.[key] ?? key);

env.addFilter("json_attr", (value) =>
  JSON.stringify(value ?? null).replace(/"/g, "&quot;"),
);

// فلاتر إضافية عشان عدد المسجَّلين يقرب من الحقيقي
for (let i = 0; i < 90; i++) {
  env.addFilter(`noop_${i}`, (value) => value);
}

export function render(template, data) {
  return new Promise((resolve, reject) => {
    env.render(template, data, (err, res) => (err ? reject(err) : resolve(res)));
  });
}
