use serde::{Deserialize, Serialize};

// ── Phone → Server ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Player ต้องการ join game
    Join { name: String },

    /// Joystick + fire state (ส่งต่อเนื่อง ~30Hz)
    Input {
        /// Joystick axis: x ∈ [-1, 1], y ∈ [-1, 1]
        joy: [f32; 2],
        fire: bool,
    },

    /// ซื้อ item ใน market
    Buy { item: String },
}

// ── Server → Phone ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    /// ยืนยัน join สำเร็จ + slot ที่ได้รับ
    Joined { player_id: u8, slot: u8 },

    /// Server เต็มแล้ว (8 คน)
    Full,

    /// Game state ส่งให้ phone ทุก frame
    State {
        hp: u8,
        max_hp: u8,
        score: u32,
        coins: u32,
        weapon_level: u8,
        respawning: bool,
        respawn_seconds: f32,
    },

    /// Event notification (wave start, item pickup, etc.)
    Event { msg: String },

    /// Market phase เริ่ม — ส่ง item list ให้ phone แสดง
    Market { items: Vec<MarketItem> },
}

#[derive(Debug, Serialize, Clone)]
pub struct MarketItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
}

// ── Shared: Game ← Server ───────────────────────────────────────────────────

/// Input จาก phone หนึ่งคน ส่งผ่าน channel เข้า game loop
#[derive(Debug, Clone)]
pub struct PlayerInput {
    pub player_id: u8,
    pub joy: [f32; 2],
    pub fire: bool,
    /// Some("item_id") เมื่อผู้เล่นซื้อ item ใน market
    pub buy_item: Option<String>,
    /// Some("name") เมื่อ phone ส่ง Join message (ครั้งแรก)
    pub name: Option<String>,
    /// true = phone disconnect แล้ว → ลบออกจาก game
    pub disconnect: bool,
}

/// Event จาก game ออกไป server เพื่อ broadcast หรือ unicast
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// State ให้ player คนนี้ (ส่งทุก frame)
    PlayerState {
        player_id: u8,
        hp: u8,
        max_hp: u8,
        score: u32,
        coins: u32,
        weapon_level: u8,
        respawning: bool,
        respawn_seconds: f32,
    },
    /// Broadcast event message ให้ทุกคน
    Broadcast(String),
    /// ส่ง market offers ให้ player คนนี้ (ครั้งเดียวตอน Market phase เริ่ม)
    SendMarket { player_id: u8, items: Vec<MarketItem> },
}
