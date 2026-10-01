use async_graphql::{Error, ErrorExtensions};
use kernel::DomainError;

fn to_gql(err: DomainError, hide_internals: bool) -> Error {
    let code = err.code();
    let message = if hide_internals && !err.is_public() {
        "internal error".to_owned()
    } else {
        err.to_string()
    };

    Error::new(message).extend_with(|_, e| e.set("code", code))
}

/// بيحوّل `Result<T, DomainError>` لنتيجة جراف‑كيو‑إل.
pub trait DomainResultExt<T> {
    /// للبوابات الموثوقة (admin / partner) — بيطلّع الرسالة كاملة.
    fn gql(self) -> Result<T, Error>;

    /// للبوابة العامة — بيخفي تفاصيل الأخطاء الداخلية.
    fn gql_public(self) -> Result<T, Error>;
}

impl<T> DomainResultExt<T> for Result<T, DomainError> {
    fn gql(self) -> Result<T, Error> {
        self.map_err(|e| to_gql(e, false))
    }

    fn gql_public(self) -> Result<T, Error> {
        self.map_err(|e| to_gql(e, true))
    }
}
