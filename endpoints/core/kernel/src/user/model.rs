use crate::{Money, UserId};

/// الموديل الكامل — فيه كل الحقول حتى الحسّاسة.
/// كل بوابة بتختار هي تعرض إيه منه، والدومين مش شغله ده.
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    /// admin بس اللي بيشوفه
    pub salary: Money,
    /// admin بس اللي بيشوفه
    pub internal_notes: String,
    pub banned: bool,
}

/// إنبوت الإنشاء — مفصول عن الموديل عشان الـ id بيتولّد جوّه.
#[derive(Debug, Clone)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub salary: Money,
}
