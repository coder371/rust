# 07 — Collections و Iterators و Closures

📄 الكود: [`examples/07_iterators.rs`](../examples/07_iterators.rs)

## JS eager مقابل Rust lazy

```js
orders.filter(o => o.paid).map(o => o.total).slice(0, 2)
// ↑ filter بيعمل array جديد كامل، map بيعمل array تاني كامل، slice تالت
```
```rust
orders.iter().filter(|o| o.paid).map(|o| o.total).take(2).collect::<Vec<_>>()
// ↑ مفيش ولا array متعمل لحد collect، وبيقف بعد أول 2 نتايج
// والـ compiler بيحوّلها لـ for loop واحدة
```

## جدول الترجمة

| JS | Rust |
|---|---|
| `arr.map(f)` | `.iter().map(f).collect()` |
| `arr.filter(f)` | `.iter().filter(f)` |
| `arr.reduce(f, init)` | `.fold(init, f)` أو `.sum()` / `.product()` |
| `arr.find(f)` | `.find(f)` → `Option` |
| `arr.findIndex(f)` | `.position(f)` → `Option<usize>` |
| `arr.some(f)` / `every(f)` | `.any(f)` / `.all(f)` |
| `arr.includes(x)` | `.contains(&x)` |
| `arr.forEach(f)` | `for x in &arr { }` (الأفضل) أو `.for_each(f)` |
| `arr.flatMap(f)` | `.flat_map(f)` |
| `arr.slice(a, b)` | `&arr[a..b]` أو `.skip(a).take(b - a)` |
| `arr.entries()` | `.enumerate()` |
| `arr.sort((a,b) => ...)` | `.sort_by(\|a, b\| ...)` / `.sort_by_key(\|x\| ...)` |
| `[...new Set(arr)]` | `.collect::<HashSet<_>>()` |
| `Object.groupBy` | `HashMap` + `entry().or_insert()` |
| `arr.length` | `.len()` |

## iter / iter_mut / into_iter

| | بيدّيك | الـ collection بعدها |
|---|---|---|
| `.iter()` | `&T` | لسه بتاعتك |
| `.iter_mut()` | `&mut T` | اتعدلت |
| `.into_iter()` | `T` | راحت (moved) |

## Closures

زي arrow functions. الفرق الوحيد المهم: `move`.

```rust
let name = String::from("x");
let c = move || println!("{name}");   // الـ closure خد ownership الـ name
```

هتحتاج `move` دايماً مع `thread::spawn` و `tokio::spawn`، لأن الـ task ممكن تعيش أطول من الـ function اللي عملتها.

## جرّب بنفسك

1. من `orders()`: هات أكبر order لكل customer (`HashMap<String, Order>`).
2. هات أسماء الـ customers اللي عندهم أكتر من order واحد، مترتبين أبجدياً.
3. اكتب word counter: `fn word_freq(text: &str) -> Vec<(String, usize)>` مترتب تنازلي بعدد المرات.
