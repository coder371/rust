# 10 — Concurrency: Threads و Arc و Mutex

📄 الكود: [`examples/10_concurrency.rs`](../examples/10_concurrency.rs)

## Node مقابل Rust

| | Node | Rust |
|---|---|---|
| الـ JS/الكود بتاعك بيشتغل على | thread واحد | أي عدد threads |
| CPU parallelism | `worker_threads` (معزولة، بتنسخ الداتا) | `std::thread` / tokio / rayon — نفس الـ memory |
| مشاركة الداتا | `postMessage` (نسخ) | `Arc<T>` (مشاركة من غير نسخ) |
| الحماية من الـ race conditions | مفيش مشكلة (thread واحد) | الـ compiler بيمنعها |

## "Fearless Concurrency"

في Java/C++/Go ممكن تكتب كود فيه data race وهيعمل compile ويشتغل... لحد ما يبوظ في production.

في Rust:
```rust
let counter = Rc::new(RefCell::new(0));
thread::spawn(move || *counter.borrow_mut() += 1);
// ❌ `Rc<RefCell<i32>>` cannot be sent between threads safely
```
الـ compiler بيرفض. لازم تستخدم الأدوات الصح:

| الأداة | الاستخدام |
|---|---|
| `Arc<T>` | مشاركة read-only بين threads |
| `Arc<Mutex<T>>` | مشاركة + تعديل (واحد في المرة) |
| `Arc<RwLock<T>>` | قراية كتير / كتابة نادرة (cache, config) |
| `AtomicU64` وأخواته | عدادات وflags بسيطة (أسرع من Mutex) |
| `mpsc::channel` | بدل المشاركة: ابعت رسايل (زي postMessage) |

## Rc مقابل Arc

- `Rc` = reference counted (عدّاد عادي). thread واحد بس. أسرع.
- `Arc` = Atomic Rc. آمن بين threads. أبطأ سنة.

الـ compiler هيقولك لو استخدمت `Rc` في مكان محتاج `Arc`. مش هتغلط.

## في الـ backend: غالباً مش هتستخدم threads مباشرة

هتستخدم **tokio** (الدرس 11) وهو بيوزّع الـ tasks على الـ threads لوحده. بس نفس المفاهيم (`Arc`, `Mutex`, `Send`) هتقابلها هناك — مثلاً الـ shared state في web server:

```rust
struct AppState { db: PgPool, cache: RwLock<HashMap<String, String>> }
let state = Arc::new(AppState { ... });   // كل request بياخد clone من الـ Arc
```

## جرّب بنفسك

1. غيّر `Arc` لـ `Rc` في `shared_state()` واقرا الـ error.
2. اكتب برنامج بيقسم `Vec` فيه 1,000,000 رقم على 4 threads، كل thread يحسب مجموع جزء، والـ main يجمع النتايج.
3. اعمل نفس الكلام بـ channel بدل `join`.
