/**
 * بدائل مونجو وريدس.
 *
 * الانتظار متعمّد إنه `setTimeout` مش حلقة انشغال: ده بالظبط شكل الـ I/O
 * الحقيقي — بيسيب الـ event loop فاضي لطلبات تانية. اللي بيحدّد السقف في
 * Node هو زمن المعالج، مش زمن الانتظار.
 *
 * أحجام الكائنات مقصودة كمان: `.lean({ virtuals: true })` مع populate
 * متداخل في core-service بيرجّع شجرة كبيرة، وبناءها ونسخها شغل معالج حقيقي.
 */
import { config } from "./config.js";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

let mongoOps = 0;
let redisOps = 0;

export const counters = {
  reset() {
    mongoOps = 0;
    redisOps = 0;
  },
  snapshot: () => ({ mongoOps, redisOps }),
};

export async function mongoFind(build) {
  mongoOps += 1;
  await sleep(config.mongoMs);
  return build();
}

export async function redisGet(build) {
  redisOps += 1;
  await sleep(config.redisMs);
  return build ? build() : null;
}

export async function redisSet() {
  redisOps += 1;
  await sleep(config.redisMs);
}

// ---------------------------------------------------------------
//  مولّدات بيانات بأحجام واقعية
// ---------------------------------------------------------------

const oid = (seed) => seed.toString(16).padStart(24, "0");

export function makeProduct(index) {
  return {
    _id: oid(index + 1000),
    handle: `product-${index}`,
    title: `منتج تجريبي رقم ${index}`,
    description:
      "وصف المنتج بالتفصيل، فيه كلام كفاية عشان يشبه المحتوى الحقيقي اللي بيتخزّن في الداتابيز ويتبعت للقالب.",
    status: "active",
    onlineStoreOnly: true,
    price: 100 + index,
    compareAtPrice: 150 + index,
    currency: "EGP",
    images: Array.from({ length: 4 }, (_, i) => ({
      _id: oid(index * 10 + i),
      url: `https://cdn.example.com/media/store/product-${index}-${i}.webp`,
      alt: `صورة ${i} للمنتج ${index}`,
      width: 1200,
      height: 1200,
    })),
    variants: Array.from({ length: 3 }, (_, i) => ({
      _id: oid(index * 100 + i),
      title: `مقاس ${i}`,
      sku: `SKU-${index}-${i}`,
      price: 100 + index + i,
      inventory: { quantity: 25 - i, policy: "deny" },
      options: [{ name: "المقاس", value: `${38 + i}` }],
    })),
    options: [{ name: "المقاس", values: ["38", "39", "40"] }],
    collections: [oid(index + 5000)],
    seo: { title: `منتج ${index}`, description: "وصف السيو" },
    createdAt: "2026-01-01T00:00:00.000Z",
  };
}

export const makeProductList = (count) =>
  Array.from({ length: count }, (_, i) => makeProduct(i));

/**
 * أنواع الحقول زي المسجّلة في FieldProcessorService.
 * البسيطة مالهاش استعلام؛ اللي بتجيب كيانات هي اللي بتضرب الداتابيز.
 */
const SIMPLE_TYPES = ["text", "color", "richtext", "url", "number", "checkbox"];

/**
 * ملامح الويدجت.
 *
 * قسم حقيقي في الثيم عنده إعدادات كتير، بس واحد أو اتنين بس منها بيجيبوا
 * كيانات. خلط الاتنين هو اللي بيحدّد عدد الاستعلامات في الصفحة.
 */
export const WIDGET_PROFILES = {
  /** بانر/نص/فيديو — مفيش استعلامات */
  static: { simple: 8, entityTypes: [] },
  /** شبكة منتجات — قائمة واحدة */
  productGrid: { simple: 6, entityTypes: ["products"] },
  /** منتجات قسم معيّن */
  collectionGrid: { simple: 6, entityTypes: ["collection_products"] },
  /** هيدر بقائمة تنقّل */
  navigation: { simple: 5, entityTypes: ["menu"] },
  /** منتج مميّز واحد */
  featured: { simple: 6, entityTypes: ["product", "media"] },
};

export function makeWidget(index, profileName = "static") {
  const profile = WIDGET_PROFILES[profileName] ?? WIDGET_PROFILES.static;
  const settings = {};
  const data = {};
  let field = 0;

  for (let i = 0; i < profile.simple; i++, field++) {
    settings[`field_${field}`] = { type: SIMPLE_TYPES[i % SIMPLE_TYPES.length], default: null };
    data[`field_${field}`] = `قيمة نصية للحقل رقم ${field} في القسم ${index}`;
  }

  for (const type of profile.entityTypes) {
    settings[`field_${field}`] = { type, default: null };
    data[`field_${field}`] = Array.from({ length: 6 }, (_, k) => oid(k + 200));
    field++;
  }

  return {
    _id: oid(index + 100),
    name: `widget-${index}`,
    widgetKey: `section-${index}`,
    profile: profileName,
    settings,
    data,
    blocks: [],
  };
}

/** بلوك: حقول بسيطة بس — زي شريحة سلايدر أو عنصر قائمة */
export function makeBlock(index) {
  const settings = {};
  const data = {};

  for (let i = 0; i < 4; i++) {
    settings[`field_${i}`] = { type: SIMPLE_TYPES[i % SIMPLE_TYPES.length], default: null };
    data[`field_${i}`] = `نص البلوك ${index} حقل ${i}`;
  }

  return { _id: oid(index + 900), name: `block-${index}`, widgetKey: `block-${index}`, settings, data };
}
