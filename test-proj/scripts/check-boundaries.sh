#!/usr/bin/env bash
# فحص قواعد العزل. يُشغَّل في الـ CI — القاعدة التي لا تُفحَص آلياً هي أمنية لا قاعدة.
set -uo pipefail
cd "$(dirname "$0")/.."
fail=0
ok()   { printf '  \033[32m✓\033[0m %s\n' "$1"; }
bad()  { printf '  \033[31m✗\033[0m %s\n' "$1"; fail=1; }

echo "١) لا موديول يعتمد على موديول آخر"
for m in crates/modules/*/; do
  name="qumra-$(basename "${m%/}")"
  deps=$(cargo tree -p "$name" -e normal --depth 1 --prefix none 2>/dev/null \
         | grep -oE '^qumra-[a-z-]+' | grep -v "^$name\$" \
         | grep -vE '^qumra-(kernel|contracts|platform)$' | sort -u)
  if [ -z "$deps" ]; then ok "$name"; else bad "$name يعتمد على: $(echo $deps)"; fi
done

echo "٢) الـ domain لا يستورد أي تقنية"
hits=$(grep -rlE '\b(mongodb|redis|lapin|reqwest|axum|async_graphql)\b' crates/modules/*/src/domain/ 2>/dev/null || true)
if [ -z "$hits" ]; then ok "كل طبقات domain نظيفة"; else bad "تسرّب تقني في: $hits"; fi

echo "٣) الـ graphql لا يرى الـ infrastructure"
hits=$(grep -rln 'infrastructure::' crates/modules/*/src/graphql/ 2>/dev/null || true)
if [ -z "$hits" ]; then ok "كل طبقات graphql نظيفة"; else bad "تسرّب في: $hits"; fi

echo "٤) لا استعلام تخزين بلا store_id"
hits=$(grep -rn 'find_one(doc! {\|find(doc! {' crates/modules/*/src/infrastructure/ 2>/dev/null \
       | grep -v store_id || true)
if [ -z "$hits" ]; then ok "كل الاستعلامات مقيّدة بالمتجر"; else bad "استعلام غير مقيّد: $hits"; fi

echo "٥) الأسلاك بين الموديولات في جذر التركيب وحده"
hits=$(grep -rln 'qumra_inventory\|qumra_notifications' crates/modules/ 2>/dev/null || true)
if [ -z "$hits" ]; then ok "لا موديول يذكر موديولاً آخر"; else bad "خرق في: $hits"; fi

[ $fail -eq 0 ] && echo "الحدود سليمة." || echo "فشل فحص الحدود."
exit $fail
