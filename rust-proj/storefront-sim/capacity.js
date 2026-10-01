/**
 * من ميزانية الإعلانات إلى طلبات في الثانية.
 *
 * كل الافتراضات في `ASSUMPTIONS` — غيّرها من البيئة وشوف الأثر:
 *   CPC=1.5 node capacity.js
 */
const num = (key, fallback) => {
  const v = Number(process.env[key]);
  return Number.isFinite(v) ? v : fallback;
};

const ASSUMPTIONS = {
  merchants: num("MERCHANTS", 200),
  dailyBudgetEgp: num("DAILY_BUDGET", 1000),
  /** تكلفة النقرة بالجنيه — أكبر مصدر عدم يقين، فبنعرض نطاق */
  cpcEgp: num("CPC", 2),
  /** صفحات لكل جلسة زائر */
  pagesPerSession: num("PAGES_PER_SESSION", 3.5),
  /** نسبة الزيارات الجاية من الإعلانات (الباقي عضوي/مباشر) */
  adTrafficShare: num("AD_SHARE", 0.6),
  /** نسبة زيارات اليوم اللي بتتركّز في ساعة الذروة */
  peakHourShare: num("PEAK_HOUR_SHARE", 0.12),
  /** تفاوت داخل ساعة الذروة نفسها */
  burstFactor: num("BURST_FACTOR", 2),
};

/** المقيس من storefront-sim على ٦ أنوية */
const MEASURED = {
  asisPerCore: 34.9,
  asisSixCores: 169.1,
  tunedPerCore: 861.2,
  tunedSixCores: 2570.0,
};

function capacity(a = ASSUMPTIONS) {
  const totalBudget = a.merchants * a.dailyBudgetEgp;
  const clicksPerDay = totalBudget / a.cpcEgp;
  const adPageViews = clicksPerDay * a.pagesPerSession;
  const totalPageViews = adPageViews / a.adTrafficShare;

  const avgRps = totalPageViews / 86_400;
  const peakHourRps = (totalPageViews * a.peakHourShare) / 3_600;
  const burstRps = peakHourRps * a.burstFactor;

  return { totalBudget, clicksPerDay, adPageViews, totalPageViews, avgRps, peakHourRps, burstRps };
}

const fmt = (n, d = 0) => n.toLocaleString("en-US", { maximumFractionDigits: d });

console.log("═".repeat(72));
console.log("  الطلب: من ميزانية الإعلانات إلى حمل الخادم");
console.log("═".repeat(72));
const base = capacity();
console.log(`  ${ASSUMPTIONS.merchants} تاجر × ${fmt(ASSUMPTIONS.dailyBudgetEgp)} ج/يوم = ${fmt(base.totalBudget)} ج/يوم`);
console.log(`  عند ${ASSUMPTIONS.cpcEgp} ج للنقرة → ${fmt(base.clicksPerDay)} نقرة/يوم`);
console.log(`  × ${ASSUMPTIONS.pagesPerSession} صفحة/جلسة → ${fmt(base.adPageViews)} مشاهدة من الإعلانات`);
console.log(`  ÷ ${ASSUMPTIONS.adTrafficShare} (الإعلانات ${ASSUMPTIONS.adTrafficShare * 100}% من الزيارات) → ${fmt(base.totalPageViews)} مشاهدة/يوم إجمالاً`);
console.log();
console.log(`  المتوسط:        ${fmt(base.avgRps, 1)} طلب/ث`);
console.log(`  ساعة الذروة:    ${fmt(base.peakHourRps, 1)} طلب/ث`);
console.log(`  ذروة الذروة:    ${fmt(base.burstRps, 1)} طلب/ث   ← ده اللي لازم نستحمله`);
console.log();

console.log("═".repeat(72));
console.log("  حساسية تكلفة النقرة (أهم افتراض)");
console.log("═".repeat(72));
console.log("  CPC     نقرات/يوم    مشاهدات/يوم    ذروة طلب/ث   يكفيها الحالي؟   بعد الإصلاح؟");
for (const cpc of [1, 1.5, 2, 3, 5]) {
  const c = capacity({ ...ASSUMPTIONS, cpcEgp: cpc });
  const okNow = c.burstRps <= MEASURED.asisSixCores;
  const okFixed = c.burstRps <= MEASURED.tunedSixCores;
  console.log(
    `  ${String(cpc).padStart(4)} ج  ${fmt(c.clicksPerDay).padStart(10)}  ${fmt(c.totalPageViews).padStart(13)}  ` +
    `${fmt(c.burstRps, 1).padStart(11)}   ${(okNow ? "أيوه" : "لأ").padStart(12)}   ${(okFixed ? "أيوه" : "لأ").padStart(11)}`,
  );
}
console.log();

console.log("═".repeat(72));
console.log("  الطاقة المقيسة (٦ أنوية)");
console.log("═".repeat(72));
const headroomNow = MEASURED.asisSixCores / base.burstRps;
const headroomFixed = MEASURED.tunedSixCores / base.burstRps;
console.log(`  الكود الحالي:    ${fmt(MEASURED.asisSixCores, 1).padStart(7)} طلب/ث   هامش ×${fmt(headroomNow, 1)}`);
console.log(`  بعد الإصلاح:     ${fmt(MEASURED.tunedSixCores, 1).padStart(7)} طلب/ث   هامش ×${fmt(headroomFixed, 1)}`);
console.log();
console.log(`  أقصى عدد تجّار بنفس الميزانية:`);
console.log(`    بالكود الحالي:  ${fmt(ASSUMPTIONS.merchants * headroomNow)} تاجر`);
console.log(`    بعد الإصلاح:    ${fmt(ASSUMPTIONS.merchants * headroomFixed)} تاجر`);
