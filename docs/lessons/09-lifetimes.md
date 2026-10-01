# 09 — Lifetimes

📄 الكود: [`examples/09_lifetimes.rs`](../examples/09_lifetimes.rs)

## الفكرة ببساطة

الـ lifetime (`'a`) مش بيغيّر أي حاجة وقت التشغيل. ده **توثيق للـ compiler** بيقول:
> "الـ reference اللي راجع ده عايش قد الـ reference اللي داخل ده".

الهدف: يمنع **dangling references**. في JS المشكلة دي مش موجودة لأن الـ GC مش بيمسح حاجة لسه حد بيشاور عليها. Rust بتحقق نفس الأمان **من غير GC** — وده تمنه.

## إمتى هتكتبها؟

- **نادراً.** الـ compiler بيستنتجها في الحالات العادية (elision).
- لما function **ترجع reference** وعندها **أكتر من reference داخل**.
- لما **struct يمسك reference** جواه.

## النصيحة العملية

وإنت بتتعلم وبتكتب backend:
- الـ structs → استخدم **owned types** (`String`, `Vec<T>`). مفيش lifetimes.
- الـ function params → `&str`, `&[T]`. الـ elision هيكفيك.
- لو لقيت نفسك بتكتب `'a` في كل حتة → غالباً محتاج owned data أو `Arc`.

الـ lifetimes مهمة جداً لما تكتب parsers أو libraries عالية الأداء (zero-copy). في الـ web services العادية هتقابلها قليل.

## `'static`

يعني "عايش طول البرنامج". هتقابله في:
- string literals: `&'static str`
- `tokio::spawn` بيطلب `'static` → يعني "متبعتش references لحاجات local، ابعت owned data" (عشان كده `move`).
