use serde::{Deserialize, Serialize};

// ── Ship Designer types ───────────────────────────────────────────────────────

/// หนึ่ง cell ในกริดของยาน  (col, row เริ่มจาก 0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipCell {
    pub col: u8,
    pub row: u8,
    pub part: ShipPart,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipPart {
    Hull,
    Cockpit,
    Engine,
    Weapon,
    Wing,
}

/// ข้อมูลยานที่ออกแบบ ส่งจาก phone → server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipDesign {
    pub cells: Vec<ShipCell>,
}

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

    /// ส่ง ship design ที่ออกแบบไว้ใน Lobby
    ShipDesign { cells: Vec<ShipCell> },
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
    /// ship design ที่ player ออกแบบไว้ใน Lobby
    pub ship_design: Option<ShipDesign>,
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

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // ── ClientMsg: deserialize from JSON (untrusted network boundary) ──────────

    #[test]
    fn deserialize_join() {
        let msg: ClientMsg =
            serde_json::from_str(r#"{"type":"join","name":"Alice"}"#).unwrap();
        match msg {
            ClientMsg::Join { name } => assert_eq!(name, "Alice"),
            other => panic!("expected Join, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_input() {
        let msg: ClientMsg = serde_json::from_str(
            r#"{"type":"input","joy":[0.5,-0.3],"fire":true}"#,
        )
        .unwrap();
        match msg {
            ClientMsg::Input { joy, fire } => {
                assert_eq!(joy, [0.5, -0.3]);
                assert!(fire);
            }
            other => panic!("expected Input, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_buy() {
        let msg: ClientMsg =
            serde_json::from_str(r#"{"type":"buy","item":"weapon_up"}"#).unwrap();
        match msg {
            ClientMsg::Buy { item } => assert_eq!(item, "weapon_up"),
            other => panic!("expected Buy, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_ship_design() {
        let msg: ClientMsg = serde_json::from_str(
            r#"{"type":"ship_design","cells":[{"col":3,"row":1,"part":"cockpit"}]}"#,
        )
        .unwrap();
        match msg {
            ClientMsg::ShipDesign { cells } => {
                assert_eq!(cells.len(), 1);
                assert_eq!(cells[0].col, 3);
                assert_eq!(cells[0].row, 1);
                assert_eq!(cells[0].part, ShipPart::Cockpit);
            }
            other => panic!("expected ShipDesign, got {other:?}"),
        }
    }

    #[test]
    fn deserialize_rejects_unknown_type() {
        let result: Result<ClientMsg, _> =
            serde_json::from_str(r#"{"type":"foo","x":1}"#);
        assert!(result.is_err(), "type ที่ไม่รู้จักต้อง error");
    }

    #[test]
    fn deserialize_rejects_missing_required_field() {
        // Join ต้องมี name
        let result: Result<ClientMsg, _> =
            serde_json::from_str(r#"{"type":"join"}"#);
        assert!(result.is_err(), "field ที่จำเป็นหายไปต้อง error");
    }

    // ── ShipPart: snake_case round-trip ───────────────────────────────────────

    #[test]
    fn ship_part_snake_case_round_trip() {
        for (part, expected_str) in [
            (ShipPart::Hull, "hull"),
            (ShipPart::Cockpit, "cockpit"),
            (ShipPart::Engine, "engine"),
            (ShipPart::Weapon, "weapon"),
            (ShipPart::Wing, "wing"),
        ] {
            let s = serde_json::to_string(&part).unwrap();
            assert_eq!(s, format!("\"{expected_str}\""));
            let back: ShipPart = serde_json::from_str(&s).unwrap();
            assert_eq!(back, part);
        }
    }

    // ── ServerMsg::State: serialize shape (phone contract) ────────────────────

    #[test]
    fn state_message_serializes_all_fields_phone_relies_on() {
        let msg = ServerMsg::State {
            hp: 2,
            max_hp: 3,
            score: 1500,
            coins: 45,
            weapon_level: 2,
            respawning: false,
            respawn_seconds: 0.0,
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&msg).unwrap()).unwrap();

        assert_eq!(json["type"], "state");
        assert_eq!(json["hp"], 2);
        assert_eq!(json["max_hp"], 3);
        assert_eq!(json["score"], 1500);
        assert_eq!(json["coins"], 45);
        assert_eq!(json["weapon_level"], 2);
        assert_eq!(json["respawning"], false);
        assert_eq!(json["respawn_seconds"], 0.0);
    }
}
