//! اتصال الـ WebSocket لكل عميل.
//!
//! كل اتصال بيتقسم لمهمتين مستقلتين:
//!   - مهمة **إرسال**: بتسمع قناة البث بتاعة الغرفة وتدفع لها للعميل.
//!   - مهمة **استقبال**: بتقرا من العميل وتنشر في الغرفة.
//!
//! الفصل ده مقصود: القراءة والكتابة على السوكِت بيحصلوا في نفس الوقت،
//! فعميل بطيء في الاستقبال مايوقّفش استقبال رسائله، والعكس.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use crate::{
    auth::jwt,
    error::{AppError, AppResult},
    models::{now_millis, ChatMessage, ClientEvent, ServerEvent},
    state::{AppState, Room},
};

/// كل قد إيه نبعت ping للتأكد إن العميل لسه عايش.
const HEARTBEAT: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
pub struct WsQuery {
    /// التوكن بييجي في الـ query مش في هيدر، لأن الـ WebSocket API في
    /// المتصفح مابيسمحش بإضافة هيدرات. التحقق بيحصل قبل الترقية.
    token: String,
    room: String,
}

pub async fn ws_handler(
    upgrade: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> AppResult<Response> {
    // بنتحقق من التوكن والغرفة **قبل** الترقية: كده الرفض بيرجع كرد
    // HTTP عادي بسبب واضح، بدل ما نفتح سوكِت ونقفله بعدها.
    let claims = jwt::verify_token(&query.token, state.config.jwt_secret.as_bytes())?;

    if !state.user_exists(&claims.sub) {
        return Err(AppError::Unauthorized("المستخدم مش موجود".to_string()));
    }

    let room = state
        .room(&query.room)
        .ok_or_else(|| AppError::NotFound("الغرفة مش موجودة".to_string()))?;

    let limits = Limits {
        max_body_chars: state.config.max_message_chars,
        rate_limit: state.config.rate_limit,
        rate_window: Duration::from_secs(state.config.rate_window_secs),
    };

    Ok(upgrade.on_upgrade(move |socket| handle_socket(socket, room, claims.sub, limits)))
}

/// حدود الرسائل، منسوخة من الإعدادات عند فتح الاتصال.
#[derive(Clone, Copy)]
pub struct Limits {
    max_body_chars: usize,
    rate_limit: u32,
    rate_window: Duration,
}

async fn handle_socket(socket: WebSocket, room: Arc<Room>, username: String, limits: Limits) {
    let (mut sink, mut stream) = socket.split();

    // الاشتراك لازم يحصل **قبل** إرسال التاريخ. لو عكسنا الترتيب، أي رسالة
    // تتبعت في اللحظة اللي بينهم هتضيع من العميل ده.
    let mut receiver = room.subscribe();

    let history = ServerEvent::History {
        messages: room.history_snapshot(),
    };

    if send_event(&mut sink, &history).await.is_err() {
        return;
    }

    // قناة خاصة بالاتصال ده وحده — للأخطاء اللي تخصّه هو بس
    // (رسالة مرفوضة، تعدّي الحد) من غير ما باقي الغرفة تشوفها.
    let (private_tx, mut private_rx) = tokio::sync::mpsc::channel::<Arc<str>>(8);

    let is_first_session = room.join(&username);

    if is_first_session {
        room.publish(&ServerEvent::System {
            text: format!("{username} دخل الغرفة"),
        });
    }
    room.publish(&ServerEvent::Presence {
        users: room.member_names(),
    });

    tracing::info!(user = %username, room = %room.id, "client connected");

    // ---------- مهمة الإرسال ----------
    let mut send_task = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(HEARTBEAT);
        heartbeat.tick().await; // أول tick بيرجع فوراً، بنتخطاه

        loop {
            tokio::select! {
                incoming = receiver.recv() => match incoming {
                    Ok(payload) => {
                        if sink.send(Message::text(payload.to_string())).await.is_err() {
                            break;
                        }
                    }
                    // العميل بطيء لدرجة إنه فات رسائل. بنكمل بدل ما نقطع —
                    // فقدان رسايل أرحم من قطع الاتصال.
                    Err(RecvError::Lagged(missed)) => {
                        let warning = ServerEvent::System {
                            text: format!("اتخطّت {missed} رسالة بسبب بطء الاتصال"),
                        };
                        if send_event(&mut sink, &warning).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Closed) => break,
                },
                Some(payload) = private_rx.recv() => {
                    if sink.send(Message::text(payload.to_string())).await.is_err() {
                        break;
                    }
                },
                _ = heartbeat.tick() => {
                    if sink.send(Message::Ping(Vec::new().into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // ---------- مهمة الاستقبال ----------
    let recv_room = Arc::clone(&room);
    let recv_user = username.clone();

    let mut recv_task = tokio::spawn(async move {
        let mut limiter = RateLimiter::new(limits.rate_limit, limits.rate_window);

        while let Some(Ok(message)) = stream.next().await {
            let text = match message {
                Message::Text(text) => text,
                Message::Close(_) => break,
                // Ping/Pong بيتعاملوا تلقائياً، والـ binary مالوش معنى هنا.
                _ => continue,
            };

            let Ok(event) = serde_json::from_str::<ClientEvent>(&text) else {
                tracing::debug!(user = %recv_user, "malformed client event");
                continue;
            };

            match event {
                ClientEvent::Send { body } => {
                    let body = body.trim();

                    if body.is_empty() {
                        continue;
                    }

                    if body.chars().count() > limits.max_body_chars {
                        let max = limits.max_body_chars;
                        notify(&private_tx, format!("الرسالة أطول من {max} حرف")).await;
                        continue;
                    }

                    if !limiter.allow() {
                        notify(&private_tx, "بتبعت بسرعة — استنى شوية".to_string()).await;
                        continue;
                    }

                    recv_room.push_message(ChatMessage {
                        id: Uuid::new_v4().to_string(),
                        room_id: recv_room.id.clone(),
                        author: recv_user.clone(),
                        body: body.to_string(),
                        sent_at: now_millis(),
                    });
                }
                ClientEvent::Typing => {
                    if limiter.allow() {
                        recv_room.publish(&ServerEvent::Typing {
                            user: recv_user.clone(),
                        });
                    }
                }
                ClientEvent::Pong => {}
            }
        }
    });

    // أول مهمة تخلص معناها الاتصال انتهى — بنلغي التانية.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // ---------- التنظيف ----------
    let was_last_session = room.leave(&username);

    if was_last_session {
        room.publish(&ServerEvent::System {
            text: format!("{username} خرج من الغرفة"),
        });
    }
    room.publish(&ServerEvent::Presence {
        users: room.member_names(),
    });

    tracing::info!(user = %username, room = %room.id, "client disconnected");
}

/// بيبعت حدث لعميل واحد بعينه (مش بث للغرفة).
async fn send_event<S>(sink: &mut S, event: &ServerEvent) -> Result<(), ()>
where
    S: SinkExt<Message> + Unpin,
{
    let Ok(payload) = serde_json::to_string(event) else {
        return Err(());
    };

    sink.send(Message::text(payload)).await.map_err(|_| ())
}

/// بيبعت خطأ للعميل ده وحده عبر قناته الخاصة.
///
/// لو القناة مليانة أو مقفولة بنتجاهل — إبلاغ العميل مش أهم من إن
/// الخادم يفضل ماشي.
async fn notify(sender: &tokio::sync::mpsc::Sender<Arc<str>>, text: String) {
    let event = ServerEvent::Error { text };

    if let Ok(payload) = serde_json::to_string(&event) {
        let _ = sender.try_send(Arc::from(payload.as_str()));
    }
}

/// حدّ معدّل بسيط بنافذة ثابتة، واحد لكل اتصال.
///
/// الهدف منع عميل واحد من إغراق الغرفة. النافذة الثابتة أبسط من token
/// bucket وكافية هنا لأن الحد مش دقيق بطبعه.
struct RateLimiter {
    window_start: Instant,
    count: u32,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(limit: u32, window: Duration) -> Self {
        Self {
            window_start: Instant::now(),
            count: 0,
            limit,
            window,
        }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();

        if now.duration_since(self.window_start) >= self.window {
            self.window_start = now;
            self.count = 0;
        }

        self.count += 1;
        self.count <= self.limit
    }
}
