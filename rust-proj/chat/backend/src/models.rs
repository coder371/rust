//! نماذج البيانات وبروتوكول الـ WebSocket.
//!
//! الاتنين `ClientEvent` و `ServerEvent` بيستخدموا `#[serde(tag = "type")]`
//! يعني الرسالة على السلك شكلها `{"type":"send","body":"..."}` —
//! حقل واحد بيحدد النوع، سهل يتقرا من الفرونت بـ switch.

use serde::{Deserialize, Serialize};

/// رسالة شات مخزّنة ومُرسَلة.
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub author: String,
    pub body: String,
    /// ملي ثانية من epoch — الفرونت بيحوّلها لـ `new Date(sent_at)` على طول.
    pub sent_at: u64,
}

/// ملخّص غرفة، للعرض في القائمة.
#[derive(Debug, Clone, Serialize)]
pub struct RoomSummary {
    pub id: String,
    pub name: String,
    pub created_by: String,
    pub created_at: u64,
    pub members: usize,
}

/// اللي بييجي من العميل.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    /// إرسال رسالة للغرفة.
    Send { body: String },
    /// إشارة "بيكتب دلوقتي".
    Typing,
    /// رد على ping الخادم — بيخلّي الاتصال حيّ.
    Pong,
}

/// اللي بيروح للعميل.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    /// الرسائل السابقة، بتتبعت مرة واحدة عند الدخول.
    History { messages: Vec<ChatMessage> },
    /// رسالة جديدة.
    Message(ChatMessage),
    /// قائمة الموجودين في الغرفة دلوقتي.
    Presence { users: Vec<String> },
    /// حد بيكتب.
    Typing { user: String },
    /// إشعار من الخادم (دخول/خروج).
    System { text: String },
    /// خطأ خاص بالاتصال ده (مش بيتبعت للكل).
    Error { text: String },
}

/// الوقت الحالي بالملي ثانية.
pub fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}
