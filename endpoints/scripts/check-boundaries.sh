#!/usr/bin/env bash
# بيتأكد إن الأجزاء التلاتة ما اتخلطوش. شغّله في الـ CI.
#
#   ❶ core/      مايعرفش حد فوقه
#   ❷ services/  بيعرف الكور بس — والخدمات ماتعرفش بعض، الهَب بس هو اللي بيجمّعهم
#   ❸ api/       بيعرف الكيرنل والهَب — ومايوصلش للداتا مباشرة
set -uo pipefail
cd "$(dirname "$0")/.."

fail=0
report() { echo "❌ $1"; fail=1; }

# ══ ❶ الكور ══════════════════════════════════════════════════════
# الكيرنل نقي: مفيش جراف ولا axum ولا درايفر داتابيز
grep -qE '^(async-graphql|axum|mongodb|redis|sqlx)' core/kernel/Cargo.toml \
  && report "core/kernel بيعتمد على تكنولوجيا برّانية"

# الستور مايعرفش الطبقات اللي فوقه
grep -qE '^(svc-|service-runtime|services|gw-)' core/store/Cargo.toml \
  && report "core/store بيعتمد على الخدمات أو البوابات"

# ══ ❷ البيزنس لوجيك ═══════════════════════════════════════════════
for c in services/*/Cargo.toml; do
  # ولا خدمة بتعرف الطبقة الخارجية ولا الداتابيز
  grep -qE '^(async-graphql|async-graphql-axum|axum|store|gw-|mongodb|redis|sqlx)' "$c" \
    && report "$c بيعتمد على البوابات أو الستور"
done

# الخدمات ماتعرفش بعض — التجميع بيحصل في الهَب بس
for c in services/*/Cargo.toml; do
  [ "$c" = "services/hub/Cargo.toml" ] && continue
  grep -qE '^svc-' "$c" && report "$c بيعتمد على خدمة تانية — التجميع مكانه services/hub"
done

# ══ ❸ البوابات والـ API ═══════════════════════════════════════════
for c in api/gateways/*/Cargo.toml; do
  # البوابات ماتلمسش الداتا مباشرة
  grep -qE '^(store|mongodb|redis|sqlx)' "$c" \
    && report "$c بيوصل للداتا مباشرة — لازم يعدّي على services"

  # ولا بوابة بتنط على خدمة من غير الهَب
  grep -qE '^(svc-|service-runtime)' "$c" \
    && report "$c بينادي خدمة على طول — لازم يعدّي على services"

  # ولا بوابة بتعرف بوابة تانية
  name=$(grep -m1 '^name' "$c" | cut -d'"' -f2)
  grep -E '^gw-' "$c" | grep -v '^gw-shared' | grep -vq "^$name" \
    && report "$c بيعتمد على بوابة تانية"
done

[ $fail -eq 0 ] && echo "✅ الحدود سليمة — الأجزاء التلاتة كل واحد في حتّته"
exit $fail
