# endpoints — كور + خدمات + بوابات

المشروع متقسّم **3 أجزاء** كل واحد في فولدر لوحده، والترتيب ده هو ترتيب الاعتماد نفسه:

```
core/        ❶ الأساس        الموديلات والعقود وتنفيذها
services/    ❷ البيزنس لوجيك  خدمات مستقلة + هَب بيشغّل أكتر من واحدة في نفس الوقت
api/         ❸ البوابات والـAPI  4 بوابات + بينري واحد بيركّبهم
```

## الشجرة

```
core/
  kernel/                الموديلات + عقود المستودعات (traits) + الأخطاء + الـActor
  store/                 تنفيذ العقود (دلوقتي in-memory، ومكان مونجو جاهز)

services/
  runtime/               عقد Service + ServiceRegistry (بينادي كل الخدمات على التوازي)
  identity/              خدمة: المستخدمين + التوكنات
  orders/                خدمة: الأوردرات
  billing/               خدمة: الإيرادات والصرف
  hub/                   ServiceHub — بيمسك الخدمات كلها
    src/flows/           اليوزكيسات اللي بتلمس أكتر من خدمة (ملف لكل فلو)

api/
  gateways/
    shared/              تحويل الأخطاء + الترقيم + عرض الفلوس
    admin/  public/  partner/    بوابات جراف (إند بوينت = ملف)
    rest/                بوابة REST بنفس القواعد
  server/                البينري = composition root (المكان الوحيد اللي بيشوف store + البوابات)

scripts/check-boundaries.sh      بيتأكد إن الأجزاء التلاتة ما اتخلطوش
```

## القواعد

| الجزء | الكريت | بيعتمد على | ممنوع |
|---|---|---|---|
| ❶ | `kernel` | async-trait, thiserror | async-graphql, axum, أي داتابيز |
| ❶ | `store` | kernel | services, gw-* |
| ❷ | `service-runtime` | kernel | أي خدمة بعينها |
| ❷ | `svc-*` | kernel, service-runtime | async-graphql, axum, store, **أي خدمة تانية** |
| ❷ | `services` (الهَب) | كل الـ svc-* | async-graphql, axum, store |
| ❸ | `gw-*` | kernel, services, gw-shared | store, svc-* على طول, أي `gw-*` تاني |
| ❸ | `api` | الكل | — |

`./scripts/check-boundaries.sh` بيفشل لو أي قاعدة فيهم اتكسرت — حطّه في الـ CI.

## ❷ إزاي "أكتر من خدمة في نفس الوقت" شغّالة

كل خدمة مستقلة وماتعرفش أختها. اللي بيجمّعهم حاجتين:

**١. الـ registry** — كل خدمة بتنفّذ `Service`، والهَب بيسجّلها. بعد كده أي كود
يقدر يكلّم الخدمات كلها مرة واحدة، مش واحدة ورا التانية:

```rust
// services/runtime/src/registry.rs
pub async fn health(&self) -> Vec<ServiceHealth> {
    futures::future::join_all(self.services.iter().map(|s| s.health())).await
}
```

**٢. الفلوهات** — يوزكيس بيلمس أكتر من خدمة بيروح `services/hub/src/flows/`،
وبينادي اللي مالوش اعتماد على بعض على التوازي:

```rust
// services/hub/src/flows/account_overview.rs
let (user, orders, total_spent) = futures::try_join!(
    self.identity().by_id(actor, id),          // خدمة ١
    self.orders().for_user(id, page),          // خدمة ٢
    self.billing().user_spend(id),             // خدمة ٣
)?;
```

الوقت الكلي = أبطأ خدمة فيهم، مش مجموعهم.

الفلوهات الموجودة:

| الفلو | بيلمس | مكشوف في |
|---|---|---|
| `account_overview` | identity + orders + billing | `GET /rest/accounts/{id}` |
| `admin_dashboard` | identity + billing + orders + كل الخدمات | `{ dashboard }` في `/admin/graphql` |
| `order_customer` | orders ← identity (بالترتيب: الصلاحية الأول) | `{ orderCustomer }` في `/partner/graphql` |
| `platform_health` | كل الخدمات المسجّلة | `GET /rest/health` |

## التشغيل

```bash
cargo run -p api          # http://localhost:3000
```

| المسار | التوكن |
|---|---|
| `/public/graphql` | اختياري — من غيره بتبقى زائر |
| `/admin/graphql` | `Bearer admin-secret` |
| `/partner/graphql` | `Bearer partner-secret:<tenant>` مثال: `partner-secret:acme` |
| `/rest/users` | القراءة مفتوحة، الكتابة للأدمن |
| `/rest/accounts/{id}` | `Bearer admin-secret` |
| `/rest/health` | مفتوح |

كل بوابة جراف عندها كمان `/<gw>/playground` و `/<gw>/schema.graphql`.

```bash
# فحص كل الخدمات في نفس اللحظة
curl -s localhost:3000/rest/health
# {"status":"up","services":[{"service":"identity","status":"up","detail":"3 users"}, ...]}

# لوحة الإدارة — 3 خدمات في ريكوست واحد
curl -s -X POST localhost:3000/admin/graphql -H 'authorization: Bearer admin-secret' \
  -H 'content-type: application/json' \
  -d '{"query":"{ dashboard { usersCount revenue { paidOrders } recentOrders { id tenant } services { service status } } }"}'

# صورة حساب مجمّعة من 3 خدمات
curl -s localhost:3000/rest/accounts/u1 -H 'authorization: Bearer admin-secret'

# البوابة العامة — الاسم والـ id بس، والمحظور مش بيظهر
curl -s -X POST localhost:3000/public/graphql -H 'content-type: application/json' \
  -d '{"query":"{ users { id name } }"}'

# بوابة الشركاء — أوردرات الـ tenant بتاعه بس
curl -s -X POST localhost:3000/partner/graphql -H 'authorization: Bearer partner-secret:acme' \
  -H 'content-type: application/json' -d '{"query":"{ myOrders { id status } }"}'
```

## التستات

```bash
cargo test --workspace                 # سناب‑شوت لسكيما كل بوابة + تستات الفلوهات
UPDATE_SCHEMA=1 cargo test --workspace # لما تغيّر السكيما عن قصد
./scripts/check-boundaries.sh          # جراف الاعتماديات
```

- `api/server/tests/flows.rs` — بيتأكد إن الفلوهات فعلًا بتجمّع الخدمات صح
  وإن حدود الـtenant والصلاحيات ماتكسرش.
- `public_schema_hides_sensitive_fields` بيقع لو حد عرّض `salary` أو `email`
  في البوابة العامة بالغلط.

## إزاي تضيف

**❷ خدمة جديدة** → كريت `svc-<اسم>` في `services/`، بينفّذ `Service`،
وسطر تسجيل واحد في `ServiceHub::new`. كل البوابات بتشوفها فورًا من غير ما تتلمس.

**❷ يوزكيس على خدمة واحدة** → ملف في `services/<الخدمة>/src/` فيه
`impl super::XService { pub async fn ... }`.

**❷ يوزكيس بيلمس أكتر من خدمة** → ملف في `services/hub/src/flows/` فيه
`impl crate::ServiceHub { ... }` وبينادي الخدمات بـ`try_join!`.

**❸ إند بوينت جديد** → ملف في `api/gateways/<gw>/src/queries/` (أو `mutations/`)
فيه الستراكت + `#[Object]` + الإنبوت بتاعه، وسطر في `mod.rs` بيضيفه للـ `MergedObject`.

**❸ بوابة جديدة** → كريت في `api/gateways/`، وسطر `.nest(...)` في
`api/server/src/wiring.rs`. البوابة بتاخد `Arc<ServiceHub>` وخلاص.

**❶ موديل جديد** → فولدر في `core/kernel/src/` فيه `model.rs` + `repo.rs`،
وتنفيذ الـ repo في `core/store/`.

**❶ تبديل الداتابيز** → `mod mongo;` في `core/store/src/lib.rs` بينفّذ نفس الـ traits،
وسطرين في `wiring.rs`. الخدمات والبوابات ماتتلمسش.
