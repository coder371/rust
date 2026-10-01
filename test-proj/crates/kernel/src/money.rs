use serde::{Deserialize, Serialize};

/// عملات السوق العربي. المبالغ دائماً بالوحدة الصغرى (هللة/فلس/قرش)
/// حتى لا تتسرّب أخطاء الفاصلة العائمة إلى الحسابات المالية.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    Sar,
    Aed,
    Egp,
}

impl Currency {
    pub fn code(&self) -> &'static str {
        match self {
            Currency::Sar => "SAR",
            Currency::Aed => "AED",
            Currency::Egp => "EGP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub minor: i64,
    pub currency: Currency,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MoneyError {
    #[error("لا يمكن الجمع بين عملتين مختلفتين: {0} و {1}")]
    CurrencyMismatch(&'static str, &'static str),
    #[error("تجاوز في حساب المبلغ")]
    Overflow,
}

impl Money {
    pub fn new(minor: i64, currency: Currency) -> Self {
        Self { minor, currency }
    }

    pub fn zero(currency: Currency) -> Self {
        Self { minor: 0, currency }
    }

    pub fn add(self, other: Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch(
                self.currency.code(),
                other.currency.code(),
            ));
        }
        let minor = self
            .minor
            .checked_add(other.minor)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money { minor, currency: self.currency })
    }

    pub fn mul(self, qty: u32) -> Result<Money, MoneyError> {
        let minor = self
            .minor
            .checked_mul(i64::from(qty))
            .ok_or(MoneyError::Overflow)?;
        Ok(Money { minor, currency: self.currency })
    }
}
