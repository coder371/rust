/**
 * تشغيل بعدد الأنوية.
 *
 * Node بخيط واحد، والإنتاج بيشغّل نسخة لكل نواة (PM2/Kubernetes).
 * القياس بنسخة واحدة بيقلّل الطاقة الحقيقية بمقدار عدد الأنوية تقريباً.
 */
import cluster from "node:cluster";
import { availableParallelism } from "node:os";

const workers = Number(process.env.WORKERS) || availableParallelism();

if (cluster.isPrimary) {
  for (let i = 0; i < workers; i++) cluster.fork();
  cluster.on("exit", () => cluster.fork());
  console.log(`storefront-sim cluster: ${workers} نسخة`);
} else {
  await import("./server.js");
}
