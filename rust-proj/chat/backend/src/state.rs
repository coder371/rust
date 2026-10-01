//! الحالة المشتركة: المستخدمين والغرف.
//!
//! التخزين في الذاكرة. الوصول كله ماشي على `AppState` عشان لو اتحوّلنا
//! لداتابيز بعدين، التغيير يبقى هنا بس.
//!
//! ملاحظة مهمة: الأقفال دي `std::sync` مش `tokio::sync` — أسرع، بس ممنوع
//! نمسكها عبر `.await`. كل دالة هنا بتاخد القفل وتسيبه قبل ما ترجّع.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, RwLock},
};

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    config::Config,
    error::{AppError, AppResult},
    models::{now_millis, ChatMessage, RoomSummary, ServerEvent},
};

pub struct StoredUser {
    pub password_hash: String,
    pub created_at: u64,
}

pub struct Room {
    pub id: String,
    pub name: String,
    pub created_by: String,
    pub created_at: u64,

    /// بنبثّ JSON مُسلسَل جاهز، مش الحدث نفسه.
    ///
    /// السبب: السلسلة بتحصل **مرة واحدة** لكل رسالة بدل مرة لكل مشترك،
    /// والتوزيع بعد كده مجرد نسخ `Arc`. مع 100 شخص في الغرفة ده الفرق
    /// بين 100 عملية serialize و واحدة.
    tx: broadcast::Sender<Arc<str>>,

    history: Mutex<VecDeque<ChatMessage>>,
    /// اسم المستخدم -> عدد اتصالاته المفتوحة (تابين في المتصفح = اتنين).
    members: Mutex<HashMap<String, usize>>,
    history_limit: usize,
}

impl Room {
    fn new(name: String, created_by: String, history_limit: usize, capacity: usize) -> Self {
        // لو عميل بطيء اتأخّر أكتر من السعة دي، بيستقبل `Lagged` —
        // بنعالجها بإشعار بدل ما نقفل الاتصال.
        let (tx, _) = broadcast::channel(capacity);

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_by,
            created_at: now_millis(),
            tx,
            history: Mutex::new(VecDeque::with_capacity(history_limit)),
            members: Mutex::new(HashMap::new()),
            history_limit,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<str>> {
        self.tx.subscribe()
    }

    /// بيسلسل الحدث ويبعته لكل المشتركين.
    ///
    /// فشل الإرسال معناه مفيش حد مشترك حالياً — ده وضع عادي مش خطأ.
    pub fn publish(&self, event: &ServerEvent) {
        match serde_json::to_string(event) {
            Ok(payload) => {
                let _ = self.tx.send(Arc::from(payload.as_str()));
            }
            Err(error) => tracing::error!(%error, "failed to serialize server event"),
        }
    }

    /// بيخزّن الرسالة في التاريخ (بحد أقصى) وبعدين يبثّها.
    pub fn push_message(&self, message: ChatMessage) {
        {
            let mut history = self.history.lock().expect("history lock poisoned");

            if history.len() == self.history_limit {
                history.pop_front();
            }
            history.push_back(message.clone());
        }

        self.publish(&ServerEvent::Message(message));
    }

    pub fn history_snapshot(&self) -> Vec<ChatMessage> {
        self.history
            .lock()
            .expect("history lock poisoned")
            .iter()
            .cloned()
            .collect()
    }

    /// بيسجّل اتصال جديد. بيرجّع `true` لو دي أول مرة المستخدم يدخل الغرفة.
    pub fn join(&self, username: &str) -> bool {
        let mut members = self.members.lock().expect("members lock poisoned");
        let counter = members.entry(username.to_string()).or_insert(0);
        *counter += 1;
        *counter == 1
    }

    /// بيشيل اتصال. بيرجّع `true` لو دي آخر جلسة للمستخدم في الغرفة.
    pub fn leave(&self, username: &str) -> bool {
        let mut members = self.members.lock().expect("members lock poisoned");

        let Some(counter) = members.get_mut(username) else {
            return false;
        };

        *counter -= 1;

        if *counter == 0 {
            members.remove(username);
            return true;
        }
        false
    }

    pub fn member_names(&self) -> Vec<String> {
        let members = self.members.lock().expect("members lock poisoned");
        let mut names: Vec<String> = members.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn member_count(&self) -> usize {
        self.members.lock().expect("members lock poisoned").len()
    }

    pub fn summary(&self) -> RoomSummary {
        RoomSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            created_by: self.created_by.clone(),
            created_at: self.created_at,
            members: self.member_count(),
        }
    }
}

pub struct AppState {
    pub config: Config,
    users: RwLock<HashMap<String, StoredUser>>,
    rooms: RwLock<HashMap<String, Arc<Room>>>,
}

impl AppState {
    pub fn new(config: Config) -> Arc<Self> {
        let state = Self {
            config,
            users: RwLock::new(HashMap::new()),
            rooms: RwLock::new(HashMap::new()),
        };

        // غرفة افتراضية عشان الفرونت يلاقي حاجة من أول تشغيل.
        let general = Room::new(
            "عام".to_string(),
            "system".to_string(),
            state.config.history_limit,
            state.config.broadcast_capacity,
        );
        state
            .rooms
            .write()
            .expect("rooms lock poisoned")
            .insert(general.id.clone(), Arc::new(general));

        Arc::new(state)
    }

    // ---------- المستخدمين ----------

    pub fn insert_user(&self, username: String, password_hash: String) -> AppResult<()> {
        let mut users = self.users.write().expect("users lock poisoned");

        if users.contains_key(&username) {
            return Err(AppError::Conflict("الاسم ده محجوز".to_string()));
        }

        users.insert(
            username,
            StoredUser {
                password_hash,
                created_at: now_millis(),
            },
        );
        Ok(())
    }

    /// بيرجّع الهاش المخزّن للمستخدم، لو موجود.
    pub fn password_hash_of(&self, username: &str) -> Option<String> {
        self.users
            .read()
            .expect("users lock poisoned")
            .get(username)
            .map(|user| user.password_hash.clone())
    }

    pub fn user_created_at(&self, username: &str) -> Option<u64> {
        self.users
            .read()
            .expect("users lock poisoned")
            .get(username)
            .map(|user| user.created_at)
    }

    pub fn user_exists(&self, username: &str) -> bool {
        self.users
            .read()
            .expect("users lock poisoned")
            .contains_key(username)
    }

    // ---------- الغرف ----------

    pub fn create_room(&self, name: String, created_by: String) -> AppResult<RoomSummary> {
        let mut rooms = self.rooms.write().expect("rooms lock poisoned");

        if rooms.values().any(|room| room.name == name) {
            return Err(AppError::Conflict("في غرفة بنفس الاسم".to_string()));
        }

        let room = Arc::new(Room::new(
            name,
            created_by,
            self.config.history_limit,
            self.config.broadcast_capacity,
        ));
        let summary = room.summary();
        rooms.insert(room.id.clone(), room);

        Ok(summary)
    }

    pub fn room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms
            .read()
            .expect("rooms lock poisoned")
            .get(room_id)
            .cloned()
    }

    pub fn room_summaries(&self) -> Vec<RoomSummary> {
        let rooms = self.rooms.read().expect("rooms lock poisoned");
        let mut summaries: Vec<RoomSummary> = rooms.values().map(|room| room.summary()).collect();
        summaries.sort_by_key(|summary| summary.created_at);
        summaries
    }
}
