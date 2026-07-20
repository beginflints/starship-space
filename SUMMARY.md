# Starship Space — Project Summary

> เอกสารนี้สรุปสถาปัตยกรรม โครงสร้างไฟล์ และสิ่งที่ implement ไปแล้ว
> เขียนเพื่อให้ AI agent หรือนักพัฒนาคนอื่นอ่านแล้วเข้าใจ codebase ได้ทันที

---

## ภาพรวม (Overview)

เกม arcade สไตล์ Space Shooter แบบ **Local-WiFi Multiplayer** สำหรับ 1–8 ผู้เล่น

| บทบาท | อุปกรณ์ | หน้าที่ |
|--------|----------|---------|
| **Host** | Mac (หรือ PC) | รัน game + WebSocket server, แสดงผลบนจอใหญ่ |
| **Players** | มือถือ (browser ธรรมดา ไม่ต้องติดตั้ง app) | เชื่อม WiFi เดียวกัน → เปิด `http://<HOST-IP>:3000` → ได้ joystick controller ทันที |

---

## Tech Stack

| ส่วน | เทคโนโลยี | เวอร์ชัน |
|------|-----------|---------|
| Game engine | Macroquad | 0.4 |
| Async runtime | Tokio | 1 (full) |
| HTTP + WebSocket server | Axum | 0.7 (feature: ws) |
| Static file server | tower-http ServeDir | 0.5 |
| Serialization | serde + serde_json | 1 |
| Futures utilities | futures-util | **=0.3.31** (pinned!) |
| Local IP detection | local-ip-address | 0.6 |

> ⚠️ **futures-util ต้อง pin ที่ =0.3.31** เพราะ 0.3.32 มี broken dep กับ futures-macro

---

## Architecture — Threading Model

```
┌─────────────────────────────────────────────┐
│  Main Thread (OS requirement ของ Macroquad)  │
│                                             │
│  macroquad game loop (async)                │
│  game::run(input_rx, event_tx)              │
└───────────────┬─────────────────────────────┘
                │ tokio channels
      ┌─────────┴──────────┐
      │                    │
      │ mpsc::Sender        │ broadcast::Sender
      │ (phone → game)      │ (game → all phones)
      │                    │
┌─────▼────────────────────▼─────────────────┐
│  Background Thread (std::thread::spawn)     │
│                                             │
│  tokio Runtime → Axum server :3000          │
│  server::run(input_tx, event_tx_server)     │
│                                             │
│  Routes:                                    │
│    GET /ws  → WebSocket upgrade             │
│    GET /    → ServeDir("static/")           │
└─────────────────────────────────────────────┘
```

**สรุป**: ไม่มี shared memory ระหว่าง 2 threads — สื่อสารผ่าน Tokio channels เท่านั้น

---

## โครงสร้างไฟล์ (File Structure)

```
Game001/
├── Cargo.toml
├── src/
│   ├── main.rs                  ← entry point: สร้าง channels, spawn server thread, run game
│   ├── server/
│   │   ├── mod.rs               ← Axum router, AtomicU8 player counter, ws_upgrade handler
│   │   ├── ws_handler.rs        ← 1 task ต่อ player: split socket → send_task + recv_task
│   │   └── messages.rs          ← ทุก message type (ดูรายละเอียดด้านล่าง)
│   └── game/
│       ├── mod.rs               ← game loop หลัก: drain inputs → update → draw
│       ├── state.rs             ← GameState, Player, Role, WaveState, ConvoyCore, RunSummary, Particle
│       ├── bullet.rs            ← Bullet struct + spawn (weapon level 1/3/5-way spread)
│       ├── enemy.rs             ← Enemy (Basic/Fast/Tank) + build_wave_queue per wave
│       ├── item.rs              ← Item drops (HpUp/WeaponUp/CoinBoost) + roll_drop()
│       ├── market.rs            ← get_offers() + apply_purchase() ช่วง Market phase
│       ├── effects.rs           ← damage stages + weapon fx tables (data-driven)
│       ├── audio.rs             ← procedural sound effects
│       └── renderer.rs          ← Macroquad draw calls ทั้งหมด (รวม ship designer render)
└── static/
    └── index.html               ← Single-file mobile controller + ship designer (vanilla JS)
```

---

## Message Protocol (JSON over WebSocket)

### Phone → Server (`ClientMsg`)

```jsonc
{ "type": "join",  "name": "Pilot1" }            // เข้าร่วม game (ส่งตอน LAUNCH)
{ "type": "input", "joy": [0.5, -0.3], "fire": true }  // ~30Hz
{ "type": "buy",   "item": "weapon_up" }          // ซื้อ upgrade ใน Market phase
{ "type": "ship_design", "cells": [               // ส่ง ship design จาก mobile designer
    { "col": 3, "row": 1, "part": "cockpit" },
    { "col": 3, "row": 6, "part": "engine" }
  ]
}
```

### Server → Phone (`ServerMsg`)

```jsonc
{ "type": "joined", "player_id": 0, "slot": 0 }  // ยืนยัน join
{ "type": "full" }                                 // server เต็ม
{ "type": "state",
  "hp": 2, "max_hp": 3, "score": 1500, "coins": 45, "weapon_level": 2,
  "respawning": false, "respawn_seconds": 0.0      // respawn state สำหรับ overlay
}
{ "type": "event",  "msg": "Wave 3 — GO!" }        // notification
{ "type": "market", "items": [                     // เริ่ม Market phase
    { "id": "weapon_up", "name": "Weapon Lv2", "description": "...", "cost": 40 },
    { "id": "hp_restore", "name": "Repair Ship", "description": "...", "cost": 30 },
    { "id": "max_hp_up", "name": "+1 Max HP", "description": "...", "cost": 60 }
  ]
}
```


---

## Game Flow (Phase System)

```
Lobby ──── host: [M] change mode, [LEFT/RIGHT] + [Q/E] role draft
  │   phone: join → ship designer → LAUNCH
  │  [SPACE] + มีผู้เล่นอย่างน้อย 1 คนที่ ready
  ▼
Playing ──── (enemies spawn จาก pending queue ทีละตัวตาม spawn_interval)
  │              ↕ bullets, collisions, particles, item drops
  │              player ตาย → consume reinforcement → respawn หลัง 3s cooldown
  │  enemies หมด + clear_timer หมด (2.5s)
  ▼
Market ──── market_timer = 12 วินาที
  │           phone แสดงปุ่ม buy; เกมส่ง SendMarket event ครั้งเดียว (market_sent flag)
  │           ผู้เล่นซื้อ → apply_purchase() → ส่ง offers อัพเดตกลับ
  │  [ENTER] หรือ timer หมด
  ▼
Playing (wave ถัดไป)
  │
  │  ไม่มี survivor และ reinforcements = 0
  │  (หรือ convoy core แตก)
  ▼
GameOver ──── [R] to restart
             [SPACE] quick replay with same roster
             [M] change mode
             recap panel shows wave / leaders / pilot standings
```

---

## Game Systems ที่ implement แล้ว

### Player
- `id`, `name`, `connected`, `ready`, `x/y`, `vel_x/vel_y` (จาก joystick), `hp`, `max_hp`, `score`, `coins`, `weapon_level`
- `role: Role` — role ที่ host เลือกจาก lobby (ดู Role Draft ด้านล่าง)
- `firing: bool` — fire intent จาก phone input (ไม่ใช่ fire_cooldown hack)
- `invincible: f32` — invincibility window หลังโดนตี (scaling ตาม role)
- `respawn_timer: f32`, `is_respawning: bool` — respawn state หลังตาย (ดู Shared Reinforcement)
- `ship_design: Option<ShipDesign>` — custom ship layout จาก mobile designer (ดู Ship Designer)

### Role Draft (M4)
- Host เลือก player และสลับ role จาก lobby: `[LEFT/RIGHT]` เลือก player, `[Q/E]` cycle role
- Role ส่งผลกับ gameplay จริงทันทีผ่าน stat modifiers:

| Role | max HP | fire cooldown | invincibility | pickup radius | kill coin | drop coin |
|------|--------|---------------|---------------|---------------|-----------|-----------|
| `None` | 3 | 1.0× | 2.0s | 15.0 | +0 | +0 |
| `Vanguard` | 2 | 0.82× (เร็วกว่า) | 2.0s | 15.0 | +0 | +0 |
| `Guardian` | 4 | 1.0× | 2.8s | 15.0 | +0 | +0 |
| `Salvager` | 3 | 1.0× | 2.0s | 24.0 (กว้างกว่า) | +2 | +8 |

- role short label (`VGD`/`GRD`/`SLV`/`NONE`) แสดงใกล้ชื่อบน host screen

### Shared Reinforcement Mode (M3)
- ทีมมี `reinforcements` pool ร่วมกัน (`max_reinforcements = 5`)
- player ตาย:
  - ถ้า `reinforcements > 0` → ลด 1, เข้าสถานะ `is_respawning`, respawn หลัง `RESPAWN_DURATION = 3.0s` ที่จุดปลอดภัย
  - ถ้า `reinforcements == 0` → ตายถาวร (`alive = false`, ไม่ respawn)
- Game Over เกิดเมื่อ **`alive == 0 && respawning == 0`** (no-survivor rule)
- Host HUD: `Reinforcements: X/Y` มุมขวาบน + respawn countdown ใต้ชื่อผู้เล่น/ใน score bar
- Mobile: `respawning` + `respawn_seconds` ใน state message ขับ respawn overlay ที่แสดง countdown
- enemy overlap hits dedup (`dmg_list.sort_unstable(); dedup()`) ป้องกันการลด HP หลายครั้งใน frame เดียวกัน

### Ship Designer
- Mobile designer ระหว่าง join และ controller: grid 7×9, palette 5 part (`hull`, `cockpit`, `engine`, `weapon`, `wing`) + erase
- Budget cap = 20 points, part cost: hull=1, cockpit=3, engine=2, weapon=3, wing=1; cockpit cap ที่ 1 ชิ้น
- Seed default ship ให้ไม่ว่างเปล่า; design persist ระหว่าง re-open
- Launch ส่ง `join` แล้วตามด้วย `ship_design`; design ถูก render บน host (`draw_ship_from_design`) พร้อม damage overlay (`draw_ship_design_damaged`)

### Enemies (3 types)
| Type | Shape | HP | Speed | Special |
|------|-------|-----|-------|---------|
| Basic | Diamond | 2 | ช้า | ลงตรง |
| Fast | Triangle | 1 | เร็ว | ลงตรง |
| Tank | Circle | 8+ | ช้า | zigzag, แสดง HP bar |

- ทุก type: speed scale ตาม wave number
- Wave 1–3 กำหนด pattern ตายตัว, Wave 4+ escalate อัตโนมัติ

### Bullets (Weapon Levels)
| Level | Pattern |
|-------|---------|
| 1 | 1 กระสุนตรง |
| 2 | 3 กระสุน (กว้างออก) |
| 3 | 3 กระสุน (แคบกว่า) |
| 4 | 5 กระสุน |
| 5 | 5 กระสุน (กว้าง) |

### Item Drops
- ทุก enemy ที่ตาย: `roll_drop()` ใช้ hash ของ position + wave เป็น pseudo-random seed
- Drop rates: 25% CoinBoost (+20 coins), 10% HpUp (+1 hp), 5% WeaponUp (+1 level), 65% nothing
- Items bob ขึ้นลง, หายหลัง 9 วินาที, กระพริบเมื่อเหลือ < 3 วินาที

### Market Phase Upgrades
| Item ID | ผล | ราคา |
|---------|-----|------|
| `weapon_up` | weapon_level +1 (max 5) | 40/70/110/160 coins |
| `hp_restore` | ฟื้น HP เต็ม | 30 coins |
| `max_hp_up` | max_hp +1, hp +1 | 60 coins |

### Game Over Recap
- เมื่อแพ้ เกมจะ snapshot `RunSummary` ทันที
- recap panel แสดง `Reached Wave`, `Top Score`, `Coin Leader`, `Crew Status`
- ถ้าเป็น convoy run จะโชว์ `Convoy Run` และ `Core survived with X HP` หรือ `Core destroyed`
- มี `Pilot Recap` list เรียงตาม score แล้ว tie-break ด้วย coins
- กด `Space` เพื่อ quick replay โดยใช้ roster เดิม
- กด `R` เพื่อกลับ lobby โดยไม่บังคับให้ผู้เล่น join ใหม่ทันที
- กด `M` เพื่อสลับ mode แล้วกลับ lobby ทันที

---

## Debug Keys (dev build เท่านั้น)

| Key | Phase | ผล |
|-----|-------|-----|
| `D` | Lobby | เพิ่ม dummy player |
| `K` | Playing | ฆ่า enemy ทั้งหมด (skip wave) |
| `Space` | Lobby | เริ่มเกม (ต้องมีผู้เล่นอย่างน้อย 1 คนที่ ready) |
| `Space` | GameOver | quick replay (preserve roster + mode) |
| `Enter` | Market | skip Market timer |
| `R` | GameOver | return to lobby (preserve roster) |
| `M` | Lobby / GameOver | change mode (Classic ↔ Convoy) |
| `LEFT/RIGHT` | Lobby | เลื่อน role draft cursor ไปยัง player ตัวถัดไป/ก่อนหน้า |
| `Q` / `E` | Lobby | cycle role ของ player ที่เลือก (reverse / forward) |

---

## สิ่งสำคัญที่ต้องรู้ (Key Gotchas)

### 1. Macroquad ต้องรันบน Main Thread
```rust
// ต้องทำแบบนี้เสมอ — ห้ามย้าย macroquad ไป thread อื่น
std::thread::spawn(move || { tokio_runtime.block_on(server::run(...)); });
game::run(input_rx, event_tx).await; // main thread
```

### 2. futures-util ต้อง Pin Version
```toml
futures-util = "=0.3.31"  # ห้ามใช้ 0.3.32+ (futures-macro broken)
```

### 3. Borrow Checker ใน resolve_collisions
เมื่อต้องใช้ `&mut gs.enemies[i]` แล้วเรียก `gs.explode()` ต้องแยก scope:
```rust
// ✅ ถูก: ให้ borrow ของ e จบก่อน
let (ex, ey, dead, ...) = {
    let e = &mut gs.enemies[h.enemy_idx];
    e.hp -= damage;
    (e.x, e.y, e.hp <= 0, ...)  // copy values out
};
gs.explode(ex, ey, ...); // borrow ของ e จบไปแล้ว

// ❌ ผิด: e ยังอยู่ใน scope ขณะเรียก gs.explode
let e = &mut gs.enemies[i];
gs.explode(e.x, e.y, ...); // compile error
```

### 4. Color ใน Macroquad
- `Color::from_rgba(r, g, b, a)` รับ **u8** (0–255)
- `Color::new(r, g, b, a)` รับ **f32** (0.0–1.0)
- `color.r / .g / .b / .a` คือ **f32** — ต้องคูณ 255 ถ้าจะแปลงเป็น u8

### 5. market_sent Flag
ป้องกันการส่ง market offers ซ้ำ เมื่อเข้า Market phase:
```rust
if !gs.market_sent {
    gs.market_sent = true;
    // ส่ง SendMarket event ให้ทุก player
}
// reset ตอน start_next_wave → wave.start() จะ reset ให้ใน check_wave_complete
```

---

## วิธี Run

```bash
# รัน game (เปิด window + server :3000)
cargo run

# ผู้เล่นเปิด browser บนมือถือ (WiFi เดียวกัน)
# URL จะ print ใน terminal: http://<LOCAL-IP>:3000
```

---

## สถานะ Phase การพัฒนา

| Phase / Milestone | สถานะ | สิ่งที่ทำ |
|-------------------|-------|---------|
| Phase 1 | ✅ Done | WebSocket server, Mobile controller UI, Channel architecture |
| Phase 2 | ✅ Done | Bullets, Enemies, Wave system, Collision, Particles |
| Phase 3 | ✅ Done | Item drops, Market phase, Weapon/HP upgrades, Market UI บนมือถือ |
| M0 Stabilize | ✅ Done | Explicit join/disconnect lifecycle, lobby roster polish, market sync |
| M1 Convoy | ✅ Done | Convoy core + fail condition + HUD |
| M2 Game Over Recap | ✅ Done | Recap panel, quick replay, lobby return, mode switch |
| M3 Shared Reinforcement | ✅ Done | Team reinforcements pool, respawn with countdown, no-survivor game over |
| M4 Role Draft | ✅ Done | Vanguard / Guardian / Salvager roles with stat modifiers |
| Ship Designer | ✅ Done | Mobile grid editor + host rendering of custom ship layouts |
| M5 Command Relay | 🔲 Next | Commander + ping system + asymmetrical info (risky: touches protocol) |
| Polish | 🔲 Later | Sound effects polish, reconnect, high score |
