use qumra_kernel::MoneyError;

/// أخطاء عمل، لا أخطاء تقنية. لا ذكر لمونجو ولا لشبكة هنا.
#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("لا يمكن إنشاء طلب بلا أصناف")]
    EmptyOrder,
    #[error("لا يمكن خلط عملتين في طلب واحد")]
    MixedCurrency,
    #[error("كمية غير صالحة للصنف {0}")]
    InvalidQuantity(String),
    #[error("الطلب ملغي بالفعل")]
    AlreadyCancelled,
    #[error("انتقال حالة غير مسموح من {from:?}")]
    InvalidTransition { from: super::model::OrderStatus },
    #[error("الطلب غير موجود")]
    NotFound,
    #[error(transparent)]
    Money(#[from] MoneyError),
}
