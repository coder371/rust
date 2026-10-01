/**
 * تحجيم لمعدل مستهدف — الافتراضي ١٠٬٠٠٠ صفحة/ثانية.
 *
 *   TARGET=10000 node scale.js
 *   TARGET=25000 COMPRESSION=0.2 node scale.js
 *
 * كل الأرقام المقيسة جاية من نواة واحدة، عشان التوسّع يبقى على مستوى
 * الأجهزة مش الأنوية — الأجهزة المنفصلة مابتتشاركش ناقل الذاكرة ولا الشبكة.
 */
const num = (key, fallback) => {
  const v = Number(process.env[key]);
  return Number.isFinite(v) ? v : fallback;
};

const TARGET = num("TARGET", 10_000);
/** نسبة الضغط. المقيس على صفحة المحاكاة ٤.٩٪ — بس محتواها مكرر. */
const COMPRESSION = num("COMPRESSION", 0.2);
const PAGE_BYTES = num("PAGE_BYTES", 51_071);
/** أنوية الجهاز الواحد */
const CORES_PER_BOX = num("CORES_PER_BOX", 8);

/** المقيس على نواة واحدة، ٥٠ اتصال متزامن */
const MEASURED = [
  { name: "نود — الكود الحالي",  rps: 124.7, mongo: 21, mem: 336, p50: 393 },
  { name: "راست — الكود الحالي", rps: 267.6, mongo: 21, mem: 67,  p50: 185 },
  { name: "نود — بعد الإصلاح",   rps: 1551.4, mongo: 7, mem: 291, p50: 31 },
  { name: "راست — بعد الإصلاح",  rps: 1937.0, mongo: 7, mem: 9,   p50: 25 },
];

/** الكتابات لكل زائر جديد: جهاز + جلسة (init.middleware.ts) */
const WRITES_PER_VISIT = 2;
const REDIS_PER_VISIT = 4;

const fmt = (n, d = 0) => n.toLocaleString("en-US", { maximumFractionDigits: d });
const line = (c = "─") => console.log(c.repeat(74));

console.log();
line("═");
console.log(`  تحجيم لـ ${fmt(TARGET)} صفحة في الثانية`);
line("═");
console.log();

console.log("  الإعداد                     أنوية   أجهزة   ذاكرة    استعلامات مونجو/ث");
line();
for (const m of MEASURED) {
  const cores = Math.ceil(TARGET / m.rps);
  const boxes = Math.ceil(cores / CORES_PER_BOX);
  const memGb = (cores * m.mem) / 1024;
  const mongoRps = TARGET * m.mongo;
  console.log(
    `  ${m.name.padEnd(26)} ${String(cores).padStart(5)}   ${String(boxes).padStart(5)}   ` +
    `${fmt(memGb, 1).padStart(6)} GB   ${fmt(mongoRps).padStart(9)}`,
  );
}
console.log();

line("═");
console.log("  الحمل على قاعدة البيانات — ده الحائط الحقيقي");
line("═");
const tuned = MEASURED[3];
const asis = MEASURED[0];
console.log(`  بالكود الحالي:   ${fmt(TARGET * asis.mongo).padStart(9)} عملية مونجو/ث`);
console.log(`  بعد الإصلاح:     ${fmt(TARGET * tuned.mongo).padStart(9)} عملية مونجو/ث`);
console.log(`    منها كتابات:   ${fmt(TARGET * WRITES_PER_VISIT).padStart(9)} كتابة/ث  ← جهاز + جلسة لكل زائر`);
console.log(`  ريدس:            ${fmt(TARGET * REDIS_PER_VISIT).padStart(9)} عملية/ث`);
console.log();
console.log("  للمقارنة: نسخة مونجو واحدة بتستحمل ١٠–٥٠ ألف قراءة/ث،");
console.log("  والكتابات أقل بكتير. ٢٠ ألف كتابة/ث محتاجة تجزئة (sharding).");
console.log();

line("═");
console.log("  الشبكة");
line("═");
const raw = PAGE_BYTES * TARGET;
const compressed = raw * COMPRESSION;
console.log(`  خام:      ${fmt(raw / 1e6, 1).padStart(8)} ميجابايت/ث  = ${fmt((raw * 8) / 1e9, 2)} جيجابت/ث`);
console.log(`  مضغوط:    ${fmt(compressed / 1e6, 1).padStart(8)} ميجابايت/ث  = ${fmt((compressed * 8) / 1e9, 2)} جيجابت/ث   (عند ${COMPRESSION * 100}%)`);
console.log(`  تكلفة الضغط: ${fmt(0.03 * TARGET / 1000, 1)} ثانية معالج/ث عند gzip -1  ← يعني ${Math.ceil(0.03 * TARGET / 1000)} نواة زيادة`);
console.log();

line("═");
console.log("  الخلاصة");
line("═");
const asisCores = Math.ceil(TARGET / asis.rps);
const tunedCores = Math.ceil(TARGET / tuned.rps);
console.log(`  الرندر مش الحائط بعد الإصلاح: ${tunedCores} نواة بس.`);
console.log(`  الحائط هو ${fmt(TARGET * WRITES_PER_VISIT)} كتابة/ث في init.middleware.`);
console.log();
console.log(`  فرق اللغة هنا: ${asisCores - Math.ceil(TARGET / MEASURED[1].rps)} نواة بالكود الحالي،`);
console.log(`  و${Math.ceil(TARGET / MEASURED[2].rps) - tunedCores} نواة بس بعد الإصلاح.`);
console.log();
