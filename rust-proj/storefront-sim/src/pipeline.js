/**
 * مسار عرض صفحة متجر — بنفس مراحل core-service وبنفس ترتيبها.
 *
 * المرجع لكل مرحلة مكتوب جنبها عشان يبان إن المحاكاة مش مخترعة:
 *   theme-service/front/middlewares/*
 *   theme-service/render/services/get-final-widgets-for-render.service.ts
 *   theme-service/processing/widget-process/*
 *   theme-service/front/services/final-render.service.ts
 */
import { config } from "./config.js";
import {
  makeBlock,
  makeProductList,
  makeWidget,
  mongoFind,
  redisGet,
  redisSet,
} from "./fakedb.js";
import { render } from "./nunjucks.js";



/** كاش الصفحة. في وضع asis بيتكتب فيه ومحدش بيقرا منه — زي الكود الحالي. */
const pageCache = new Map();

// ---------------------------------------------------------------
//  ١. سلسلة الـ middlewares
// ---------------------------------------------------------------

/** store-context.middleware.ts — حل المتجر والدومين والدولة والسوق */
async function storeContext() {
  const store = await mongoFind(() => ({
    _id: "store-1",
    name: "متجر تجريبي",
    url: "https://example.com",
    logo: { url: "https://cdn.example.com/logo.webp", alt: "شعار", width: 400, height: 120 },
  }));

  await mongoFind(() => ({ domain: "example.com", verified: true })); // resolveCustomDomain
  await redisGet(() => ({ country: "EG" }));                          // كاش الـ GeoIP
  const market = await redisGet(() => ({ _id: "market-eg", code: "EG" }));
  const currency = { code: "EGP", rate: 1 };

  return { store, market, currency };
}

/** init.middleware.ts — الجهاز والجلسة. لاحظ: دي **كتابات** لكل زائر جديد. */
async function initVisitor() {
  await mongoFind(() => null);            // deviceModel.findOne
  await mongoFind(() => ({ _id: "d1" })); // deviceModel.create  ← كتابة
  await mongoFind(() => null);            // sessionModel.findOne
  await mongoFind(() => ({ _id: "s1" })); // sessionModel.create ← كتابة
}

/** theme.middleware.ts — الثيم وإعداداته والترجمات وسكربت البكسل */
async function themeContext() {
  const appTheme = await mongoFind(() => ({
    _id: "theme-1",
    themeVersion: { version: "1.0.0", theme: "t-1" },
    settings_data: { colors: { primary: "#111" }, typography: { base: 16 } },
    templates_settings: {
      templates_order: ["header", "main", "footer"],
    },
  }));

  const translations = await redisGet(() => ({ add_to_cart: "أضف للسلة" }));
  return { appTheme, translations };
}

// ---------------------------------------------------------------
//  ٢. معالجة الحقول — FieldProcessorService
// ---------------------------------------------------------------

/**
 * حقل واحد. الأنواع اللي بتجيب كيانات بتعمل استعلام + مرور ترجمة،
 * زي entity.processor.ts و list.processor.ts بالظبط.
 */
async function processField(type, value) {
  switch (type) {
    case "products":
    case "collections":
    case "collection_products": {
      const products = await mongoFind(() => makeProductList(config.productsPerList));
      // applyTranslationService.applyMany — مرور على كل عنصر
      return products.map((product) => ({
        ...product,
        title: `${product.title}`,
        description: `${product.description}`,
      }));
    }
    case "product":
    case "article":
    case "blog":
      return mongoFind(() => makeProductList(1)[0]);
    case "menu":
      return mongoFind(() =>
        Array.from({ length: 8 }, (_, i) => ({ title: `قسم ${i}`, url: `/c/${i}` })),
      );
    case "media":
    case "image":
      return { url: "https://cdn.example.com/media/banner.webp", alt: "بانر", width: 1600, height: 600 };
    default:
      return value;
  }
}

/** dynamic-widget-process.service.ts — بيلفّ على كل حقل */
async function processWidgetData(widget) {
  const entries = Object.entries(widget.settings);

  if (config.parallelFields) {
    // نفس النتيجة، بس الحقول المستقلة بتتنفّذ مع بعض
    const values = await Promise.all(
      entries.map(([key, setting]) => processField(setting.type, widget.data[key])),
    );
    return Object.fromEntries(entries.map(([key], i) => [key, values[i]]));
  }

  // asis: `for … of` مع await — كل حقل بيستنى اللي قبله
  const finalData = {};
  for (const [key, setting] of entries) {
    finalData[key] = await processField(setting.type, widget.data[key]);
  }
  return finalData;
}

// ---------------------------------------------------------------
//  ٣. تجميع الويدجتس — GetFinalWidgetsForRenderService
// ---------------------------------------------------------------

async function getFinalWidgets(appTheme, cacheKey) {
  // قراءة الكاش — متعلّقة في الكود الحالي (السطور ٣٩–٤٦)
  if (config.pageCache) {
    const cached = pageCache.get(cacheKey);
    if (cached) {
      await redisGet();
      return cached;
    }
  }

  // الأقسام موزّعة على header / main / footer زي الثيم الحقيقي
  const sections = config.pageSections;
  const headerCount = 1;
  const footerCount = 1;
  const mainSections = sections.slice(headerCount, sections.length - footerCount);

  // page.findOne(...).populate(appTheme → themeVersion → theme).populate(widgets, draftWidgets)
  const page = await mongoFind(() => ({
    _id: "page-1",
    handle: "index",
    title: "الرئيسية",
    description: "صفحة المتجر الرئيسية",
    path: "/",
    widgets: mainSections.map((profile, i) => makeWidget(i + 1, profile)),
  }));

  // templateModel.find({ key: { $in } }).populate(widgets, draftWidgets)
  const templates = await mongoFind(() => [
    { key: "header", _id: "tpl-header", widgets: [makeWidget(0, sections[0])] },
    {
      key: "footer",
      _id: "tpl-footer",
      widgets: [makeWidget(99, sections[sections.length - 1])],
    },
  ]);

  const all = [
    templates[0],
    { key: "main", _id: page._id, widgets: page.widgets },
    templates[1],
  ];

  const result = [];

  for (const template of all) {
    // blockModel.find({ _id: { $in } }) — استعلام واحد لكل الـ blocks
    const blocks = await mongoFind(() =>
      Array.from(
        { length: template.widgets.length * config.blocksPerWidget },
        (_, i) => makeBlock(i),
      ),
    );

    // metaobjectsResolver.resolve — تمريرة على كل مصادر البيانات
    await mongoFind(() => ({ resolved: true }));

    const widgets = [];
    let blockCursor = 0;

    for (const widget of template.widgets) {
      const data = await processWidgetData(widget);

      const widgetBlocks = [];
      for (let b = 0; b < config.blocksPerWidget; b++) {
        const block = blocks[blockCursor++];
        widgetBlocks.push({
          blockKey: block.widgetKey,
          name: block.name,
          data: await processWidgetData(block),
        });
      }

      // القالب بيقرا `products` صراحةً بدل ما يخمّن رقم الحقل —
      // الأقسام الساكنة مالهاش قائمة، فبتبقى فاضية.
      const listKey = Object.entries(widget.settings).find(([, s]) =>
        ["products", "collections", "collection_products"].includes(s.type),
      )?.[0];

      widgets.push({
        ...widget,
        template: `section-${Number(widget.name.split("-")[1]) % 14}`,
        profile: widget.profile,
        products: listKey ? data[listKey] : [],
        data,
        blocks: widgetBlocks,
      });
    }

    result.push({ key: template.key, widgets });
  }

  // الكتابة شغالة في الوضعين — في asis محدش بيقرا اللي اتكتب
  pageCache.set(cacheKey, result);
  await redisSet();

  return result;
}

// ---------------------------------------------------------------
//  ٤. الطلب كامل
// ---------------------------------------------------------------

export async function renderStorefrontPage() {
  const { store, market, currency } = await storeContext();
  await initVisitor();
  const { appTheme, translations } = await themeContext();

  const templates = await getFinalWidgets(appTheme, "store-1:index");

  // final-render.service.ts
  const html = await render("layouts/layout.njk", {
    globals: { store },
    context: { theme: appTheme },
    localization: { market, currency, language: "ar", availableLanguages: ["ar", "en"] },
    page: { handle: "index", title: "الرئيسية", description: "صفحة المتجر الرئيسية", path: "/" },
    translations,
    templates,
  });

  return html;
}
