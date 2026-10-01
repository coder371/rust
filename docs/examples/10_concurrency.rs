//! الدرس 10 — Threads و Arc و Mutex و Channels
//! شغّل:  cargo run --example 10_concurrency
//!
//! في Node: thread واحد للـ JS + event loop. لو عايز CPU parallelism → worker_threads
//!          وبتتواصل بـ postMessage (نسخ الداتا) أو SharedArrayBuffer (خطير ونادر).
//! في Rust: threads حقيقية بتشارك نفس الـ memory — وده عادة مصدر bugs رهيبة في C++/Java.
//!          لكن الـ compiler هنا بيمنع الـ data races خالص بـ traits اسمها:
//!            Send → ينفع يتنقل لـ thread تاني
//!            Sync → ينفع يتشارك (&T) بين threads
//!          اسمها "Fearless Concurrency": لو عمل compile، مفيش data race.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::Duration;

fn main() {
    basic_threads();
    shared_state();
    channels();
    scoped_threads();
}

fn basic_threads() {
    println!("── spawn + join ──");
    let data: Vec<i32> = (1..=3).collect(); // Vec على الـ heap
    // move: الـ thread بياخد ownership الـ data. من غيرها مش هيعمل compile،
    // لأن main ممكن تخلص قبل الـ thread والـ data تتمسح.
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        sum // الـ thread بيرجع قيمة
    });
    // join = await للـ thread
    println!("sum from thread = {}", handle.join().unwrap());
}

fn shared_state() {
    println!("── Arc<Mutex<T>> ──");
    // Rc  = reference counting (زي الـ GC بس بسيط) لـ thread واحد
    // Arc = Atomic Rc — ينفع بين threads
    // Mutex = يضمن إن thread واحد بس يعدّل في نفس الوقت
    //
    // جرّب تستبدل Arc بـ std::rc::Rc وشوف الـ compiler هيقول: `Rc` cannot be sent between threads safely
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for i in 0..8 {
        let counter = Arc::clone(&counter); // نسخة من الـ pointer بس (count++)، مش من الداتا
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // lock() بيرجع guard؛ لما يخرج من الـ scope الـ lock بيتفك لوحده (RAII)
                *counter.lock().unwrap() += 1;
            }
            i
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("counter = {} (لازم 8000)", *counter.lock().unwrap());

    // للعدّادات البسيطة: Atomics أسرع من Mutex
    let hits = Arc::new(AtomicU64::new(0));
    let hs: Vec<_> = (0..4)
        .map(|_| {
            let hits = hits.clone();
            thread::spawn(move || {
                hits.fetch_add(10, Ordering::Relaxed);
            })
        })
        .collect();
    hs.into_iter().for_each(|h| h.join().unwrap());
    println!("atomic hits = {}", hits.load(Ordering::Relaxed));

    // RwLock = قرّاء كتير أو كاتب واحد (مناسب لـ config/cache)
    let cache = RwLock::new(vec!["a"]);
    {
        let r1 = cache.read().unwrap();
        let r2 = cache.read().unwrap();
        println!("readers: {r1:?} {r2:?}");
    }
    cache.write().unwrap().push("b");
    println!("after write: {:?}", cache.read().unwrap());
}

fn channels() {
    println!("── channels (mpsc) ──");
    // "Don't communicate by sharing memory; share memory by communicating."
    // ده أقرب حاجة لـ postMessage في worker_threads، أو لـ queue داخلية.
    // mpsc = multi-producer, single-consumer
    let (tx, rx) = mpsc::channel::<String>();
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            for j in 0..2 {
                tx.send(format!("worker {id} → msg {j}")).unwrap();
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
    drop(tx); // لازم نقفل الـ tx الأصلي، وإلا الـ loop تحت هيستنى للأبد
    for msg in rx {
        // بيخلص لما كل الـ senders يتقفلوا
        println!("  {msg}");
    }
}

fn scoped_threads() {
    println!("── scoped threads ──");
    // thread::scope بيضمن إن كل الـ threads تخلص قبل ما الـ scope يخلص،
    // فينفع تستلف داتا من الـ stack من غير Arc ولا move
    let mut nums = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = nums.split_at_mut(3);
    thread::scope(|s| {
        s.spawn(|| left.iter_mut().for_each(|x| *x *= 10));
        s.spawn(|| right.iter_mut().for_each(|x| *x *= 100));
    });
    println!("{nums:?}");
}
