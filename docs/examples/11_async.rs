//! الدرس 11 — Async/Await مع Tokio
//! شغّل:  cargo run --example 11_async
//!
//! ═══ الفلسفة (اقرا دي كويس — فيها أكتر حاجة بتوقّع جماعة Node) ═══
//!
//! 1) Node فيها runtime (libuv + event loop) جوه اللغة. Rust مفيهاش!
//!    اللغة بتديك async/await بس، والـ runtime بتختاره إنت: tokio هو المعيار.
//!    #[tokio::main] = "شغّل event loop وحط main جواه".
//!
//! 2) Promise في JS = eager: أول ما تنادي fetch() الـ request بيبدأ فوراً.
//!    Future في Rust = lazy: مفيش أي حاجة بتحصل لحد ما تعمل .await (أو spawn).
//!    let f = fetch_user();   ← ولا حاجة حصلت لسه!
//!
//! 3) Tokio multi-threaded by default: الـ tasks بتتوزع على كل الـ CPU cores.
//!    عشان كده الداتا اللي بتدخل tokio::spawn لازم تكون Send + 'static.
//!
//! 4) أوعى تعمل blocking جوه async (std::thread::sleep، حسابات تقيلة، std::fs كبير).
//!    ده زي ما تعمل while(true) في Node — بيوقف الـ worker thread.
//!    الحل: tokio::task::spawn_blocking
//!
//! المقابل:
//!   Promise.all        → tokio::join!  /  futures::future::join_all  /  JoinSet
//!   Promise.race       → tokio::select!
//!   setTimeout(await)  → tokio::time::sleep
//!   AbortController    → select! مع cancellation token / drop للـ future
//!   EventEmitter       → tokio::sync::broadcast / mpsc

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};

async fn fetch_user(id: u32) -> String {
    sleep(Duration::from_millis(100)).await; // يمثّل network call
    format!("user-{id}")
}

async fn fetch_orders(user: &str) -> Vec<String> {
    sleep(Duration::from_millis(150)).await;
    vec![format!("{user}/order-1"), format!("{user}/order-2")]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lazy_futures().await;
    sequential_vs_concurrent().await;
    spawn_and_joinset().await;
    race_and_timeout().await;
    bounded_concurrency().await;
    channels().await;
    blocking_work().await?;
    Ok(())
}

async fn lazy_futures() {
    println!("── futures are lazy ──");
    let fut = async {
        println!("  (2) جوه الـ future");
        42
    };
    println!("  (1) عملنا الـ future، ولسه ماتنفذش");
    let v = fut.await;
    println!("  (3) value = {v}");
}

async fn sequential_vs_concurrent() {
    println!("── sequential vs concurrent ──");
    let t = Instant::now();
    let a = fetch_user(1).await;
    let b = fetch_user(2).await;
    println!("  sequential: {a}, {b} in {:?}", t.elapsed()); // ~200ms

    let t = Instant::now();
    // = await Promise.all([fetchUser(1), fetchUser(2)])
    let (a, b) = tokio::join!(fetch_user(1), fetch_user(2));
    println!("  join!:      {a}, {b} in {:?}", t.elapsed()); // ~100ms

    // وقت ما تكون العمليات بترجع Result: try_join! بتقف عند أول error (زي Promise.all بالظبط)
    let r: Result<(u8, u8), String> =
        tokio::try_join!(async { Ok(1) }, async { Err("boom".to_string()) });
    println!("  try_join!:  {r:?}");

    // Promise.all على list
    let users = futures::future::join_all((1..=3).map(fetch_user)).await;
    println!("  join_all:   {users:?}");

    // حاجات بتعتمد على بعض: الـ await بالترتيب الطبيعي
    let u = fetch_user(7).await;
    let orders = fetch_orders(&u).await;
    println!("  chained:    {orders:?}");
}

async fn spawn_and_joinset() {
    println!("── tokio::spawn / JoinSet ──");
    // spawn = task مستقلة بتشتغل في الخلفية (زي promise مش مستني عليه)
    // لازم الداتا تكون owned (move) لأن الـ task ممكن تعيش أكتر من الـ function
    let name = String::from("bg");
    let handle = tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        format!("{name} done")
    });
    println!("  {}", handle.await.unwrap()); // JoinHandle.await → Result (لو الـ task عملت panic)

    // JoinSet: مجموعة tasks وتاخد نتايجها بترتيب ما بتخلص
    let mut set = JoinSet::new();
    for (id, ms) in [(1, 120), (2, 30), (3, 70)] {
        set.spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            id
        });
    }
    while let Some(res) = set.join_next().await {
        println!("  finished task {}", res.unwrap());
    }
}

async fn race_and_timeout() {
    println!("── select! / timeout ──");
    // = Promise.race — والـ future اللي خسر بيتلغى فعلاً (dropped)، مش بيكمّل في الخلفية زي JS
    tokio::select! {
        u = fetch_user(1) => println!("  user won: {u}"),
        _ = sleep(Duration::from_millis(50)) => println!("  timer won (fetch اتلغت)"),
    }
    match timeout(Duration::from_millis(50), fetch_user(2)).await {
        Ok(u) => println!("  got {u}"),
        Err(_) => println!("  timeout!"),
    }
}

async fn bounded_concurrency() {
    println!("── bounded concurrency (زي p-limit) ──");
    let limit = Arc::new(Semaphore::new(2)); // 2 في نفس الوقت بالكتير
    let t = Instant::now();
    let mut set = JoinSet::new();
    for id in 1..=6 {
        let limit = limit.clone();
        set.spawn(async move {
            let _permit = limit.acquire_owned().await.unwrap(); // بيتفك لوحده لما يخرج من الـ scope
            fetch_user(id).await
        });
    }
    let mut n = 0;
    while let Some(r) = set.join_next().await {
        r.unwrap();
        n += 1;
    }
    println!(
        "  {n} calls in {:?} (~300ms لأن 2 بس في نفس الوقت)",
        t.elapsed()
    );

    // بديل أنضف بـ streams:
    use futures::StreamExt;
    let results: Vec<String> = futures::stream::iter(1..=4)
        .map(fetch_user)
        .buffer_unordered(2)
        .collect()
        .await;
    println!("  stream buffer_unordered: {results:?}");
}

async fn channels() {
    println!("── async channels (producer/consumer داخلي) ──");
    // bounded channel = backpressure: لو الـ consumer بطيء، الـ send بيستنى
    let (tx, mut rx) = mpsc::channel::<u32>(2);
    let producer = tokio::spawn(async move {
        for i in 1..=4 {
            tx.send(i).await.unwrap();
            println!("  sent {i}");
        }
        // tx بيتقفل هنا → الـ recv هيرجع None
    });
    while let Some(v) = rx.recv().await {
        sleep(Duration::from_millis(20)).await;
        println!("  received {v}");
    }
    producer.await.unwrap();
}

async fn blocking_work() -> anyhow::Result<()> {
    println!("── spawn_blocking ──");
    // شغل CPU تقيل أو library مش async → thread pool منفصل عشان منوقفش الـ runtime
    let sum = tokio::task::spawn_blocking(|| (1..=10_000_000u64).sum::<u64>()).await?;
    println!("  heavy sum = {sum}");
    // الملفات: tokio::fs بدل std::fs
    let exists = tokio::fs::try_exists("Cargo.toml").await?;
    println!("  Cargo.toml exists = {exists}");
    Ok(())
}
