# 11 — Async/Await مع Tokio ⭐

📄 الكود: [`examples/11_async.rs`](../examples/11_async.rs)

ده أكتر درس هيحسسك إنك في البيت — بس فيه **3 فروق** لو مفهمتهمش هتتعب.

## الفرق 1: مفيش runtime جوه اللغة

Node = JavaScript + V8 + **libuv (event loop)**. الـ event loop موجود دايماً.

Rust بتديك `async`/`await` كـ syntax بس. الـ **executor** (اللي بيشغّل الـ futures) مكتبة بتختارها. المعيار: **tokio**.

```rust
#[tokio::main]            // = "اعمل runtime وشغّل main جواه"
async fn main() { ... }
```

## الفرق 2: Futures كسولة (lazy) ⚠️

```js
// JS
const p = fetchUser(1);   // ← الـ request اتبعت فعلاً دلوقتي
// ... كود تاني ...
const u = await p;
```
```rust
// Rust
let f = fetch_user(1);    // ← ولا حاجة حصلت! ده مجرد "وصفة"
// ... كود تاني ...
let u = f.await;          // ← دلوقتي بس بيبدأ
```

**النتيجة العملية:** لو عايز حاجتين يشتغلوا مع بعض، لازم تقول كده صراحةً:

| JS | Rust |
|---|---|
| `await Promise.all([a(), b()])` | `tokio::join!(a(), b())` |
| `await Promise.all(list.map(f))` | `futures::future::join_all(list.map(f)).await` |
| Promise.all مع errors | `tokio::try_join!(...)` |
| `Promise.race([a, b])` | `tokio::select! { x = a => ..., y = b => ... }` |
| fire-and-forget | `tokio::spawn(async move { ... })` |
| p-limit / concurrency limit | `Semaphore` أو `stream.buffer_unordered(n)` |
| `setTimeout` | `tokio::time::sleep(d).await` |
| `setInterval` | `tokio::time::interval(d)` |
| timeout | `tokio::time::timeout(d, fut).await` |
| AbortController | `select!` مع shutdown signal، أو `CancellationToken` (tokio-util) |
| EventEmitter | `tokio::sync::broadcast` / `mpsc` / `watch` |

وحاجة حلوة: في `select!` الـ future اللي خسر **بيتلغى فعلاً** (بيتعمله drop). في JS الـ promise اللي خسر في `Promise.race` بيكمّل في الخلفية.

## الفرق 3: multi-threaded

tokio بيوزّع الـ tasks على **كل الـ CPU cores** (Node بتستخدم core واحد للـ JS). عشان كده:

- `tokio::spawn` بيطلب إن الـ future يكون `Send + 'static` → ابعت owned data بـ `move`.
- الـ shared state لازم `Arc` (و `Mutex` لو هتعدّل).
- في async استخدم `tokio::sync::Mutex` **لو** هتمسك الـ lock عبر `.await`. غير كده `std::sync::Mutex` أسرع وكويس.

## ⚠️ الغلطة الأخطر: blocking جوه async

```rust
async fn handler() {
    std::thread::sleep(Duration::from_secs(1));  // ❌ بيوقف الـ worker thread كله
    heavy_cpu_work();                            // ❌ نفس المشكلة
    std::fs::read_to_string("big.json");         // ⚠️ blocking I/O
}
```
زي بالظبط ما تعمل `while (true) {}` أو `fs.readFileSync` في Node request handler.

الحل:
```rust
tokio::time::sleep(d).await;                            // ✅
tokio::task::spawn_blocking(|| heavy_cpu_work()).await; // ✅ thread pool منفصل
tokio::fs::read_to_string("big.json").await;            // ✅
```

## جرّب بنفسك

1. اكتب `async fn fetch_all(ids: Vec<u32>) -> Vec<String>` تجيب كله بالتوازي بحد أقصى 3 في نفس الوقت.
2. اكتب "retry with exponential backoff": `async fn retry<F, Fut, T, E>(f: F, max: u32) -> Result<T, E>`.
3. اعمل worker pool: `mpsc::channel` بيستقبل jobs، و 3 tasks بتسحب منه وتنفذ.
