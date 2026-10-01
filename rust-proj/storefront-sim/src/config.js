/**
 * أرقام المحاكاة — كلها قابلة للضبط من البيئة.
 *
 * القيم الافتراضية مأخوذة من قراءة مسار الرندر الفعلي في core-service:
 * theme-service/front/middlewares + render/services + processing/widget-process.
 */
const num = (key, fallback) => {
  const value = Number(process.env[key]);
  return Number.isFinite(value) ? value : fallback;
};

const ALL_ON = process.env.SIM_MODE === "tuned";
const flag = (key) => ALL_ON || process.env[key] === "1";

export const config = {
  // --- شكل الصفحة ---
  /**
   * أقسام الصفحة الرئيسية — ملمح كل واحد بيحدد كام استعلام بيعمل.
   * ده شكل صفحة متجر واقعية: هيدر بقائمة، بانرات، تلات شبكات منتجات، فوتر.
   */
  pageSections: (process.env.PAGE_SECTIONS ??
    "navigation,static,productGrid,static,collectionGrid,featured,productGrid,static,static,navigation"
  ).split(",").map((s) => s.trim()).filter(Boolean),
  /** بلوكات لكل ويدجت — حقول بسيطة، من غير استعلامات */
  blocksPerWidget: num("BLOCKS_PER_WIDGET", 3),
  /** منتجات في قائمة المنتجات الواحدة */
  productsPerList: num("PRODUCTS_PER_LIST", 12),

  // --- زمن الإدخال/الإخراج (ملي ثانية) ---
  /** ذهاب وعودة لمونجو محلي */
  mongoMs: num("MONGO_MS", 0.8),
  /** ذهاب وعودة لريدس */
  redisMs: num("REDIS_MS", 0.3),

  // --- الوضع ---
  /**
   * تلات أعلام مستقلة، كل واحد بيقابل إصلاح واحد في core-service.
   * `SIM_MODE=tuned` بيولّعهم كلهم؛ وتقدر تولّع أي واحد لوحده.
   */
  njkCache: flag("NJK_CACHE"),      // كاش قوالب Nunjucks (بدل noCache: true)
  pageCache: flag("PAGE_CACHE"),    // تفعيل قراءة كاش الصفحة المتعلّقة
  parallelFields: flag("PARALLEL_FIELDS"), // معالجة الحقول بالتوازي

  get label() {
    const on = [
      this.njkCache && "njk",
      this.pageCache && "page",
      this.parallelFields && "parallel",
    ].filter(Boolean);
    return on.length ? on.join("+") : "asis";
  },

  port: num("PORT", 4100),
};
