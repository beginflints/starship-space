# Starship Space — Implementation Spec

เอกสารนี้เขียนเพื่อใช้เป็น `implementation document` สำหรับแจกงานให้ AI agents หรือ dev คนอื่นลงมือทำต่อได้ทันที

เอกสารนี้ไม่ใช่ vision doc
แต่เป็นเอกสารสำหรับตอบคำถามต่อไปนี้:

1. ตอนนี้โปรเจคอยู่ตรงไหน
2. อะไรคือ scope ที่ควรทำก่อน
3. แต่ละ milestone ต้องแก้ไฟล์ไหน
4. งานไหนแยกขนานกันได้
5. definition of done ของแต่ละ milestone คืออะไร

อ้างอิง codebase ปัจจุบัน:

- [`src/main.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/main.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/enemy.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/enemy.rs)
- [`src/game/item.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/item.rs)
- [`src/game/market.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/market.rs)
- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)
- [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- [`src/server/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/mod.rs)
- [`src/server/ws_handler.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/ws_handler.rs)
- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

เอกสารออกแบบที่เกี่ยวข้อง:

- [GAME_JOURNEY.md](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/GAME_JOURNEY.md)
- [TEAMWORK_MODES.md](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/TEAMWORK_MODES.md)
- [Plan.md](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/Plan.md)

## 1. Current Reality

## 1.1 สิ่งที่เกมมีแล้ว

- local WiFi multiplayer ผ่าน browser บนมือถือ
- host รัน game + websocket server
- player แต่ละคนคุมยานของตัวเอง
- phase loop หลัก:
  - `Lobby`
  - `Playing`
  - `Market`
  - `GameOver`
- enemy 3 แบบ:
  - `Basic`
  - `Fast`
  - `Tank`
- item drops:
  - `HpUp`
  - `WeaponUp`
  - `CoinBoost`
- market upgrades:
  - `weapon_up`
  - `hp_restore`
  - `max_hp_up`
- procedural audio module ใช้งานได้และ build ผ่าน

## 1.2 สิ่งที่ยังไม่ควรรีบทำ

ยังไม่ควรเริ่มจาก:

- commander mode แบบ asymmetrical เต็มรูปแบบ
- weekly challenge system
- codex / meta progression
- boss-heavy endgame structure

เหตุผล:

- โปรเจคยังต้องเก็บงาน core loop และ team objective ก่อน
- ถ้าเริ่มจาก feature ที่แตะทั้ง protocol + mobile UI + gameplay พร้อมกัน จะ debug ยากมาก

## 1.3 ความจริงเชิงเทคนิคที่สำคัญ

### Network contract ตอนนี้เล็กมาก

มือถือส่งได้แค่:

- `join`
- `input`
- `buy`

มือถือรับได้แค่:

- `joined`
- `state`
- `event`
- `market`

ดังนั้น feature ที่ทำได้เร็วที่สุดคือ feature ที่พึ่ง game loop และ renderer เป็นหลัก

### Player lifecycle ยังไม่แน่น

> `Flag (Claude - 2026-03-12):` items 1–3 ด้านล่างแก้แล้ว
>
> - ✅ `join lifecycle` — `ClientMsg::Join { name }` ถูก forward ผ่าน `PlayerInput.name` เข้า game แล้ว; `drain_inputs` set `p.name` และ broadcast "{name} joined!"
> - ✅ `disconnect` — `ws_handler` ส่ง `PlayerInput { disconnect: true }` หลัง socket ปิด; `drain_inputs` remove player ออกจาก vec; `server/mod.rs` decrement `player_count` Arc หลัง `handle_socket` return
> - ✅ `player name` — ชื่อจากมือถือถูกใช้ครบ join broadcast, market purchase, และ disconnect broadcast
> - ⏳ `market sync` — ยังไม่ได้ทำ (Task Card B1)

จาก [Plan.md](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/Plan.md) ยังมีปัญหาเรื่อง:

- ~~join lifecycle ยังไม่ explicit~~ ✅ แก้แล้ว
- ~~disconnect ยังไม่ถูก remove จริง~~ ✅ แก้แล้ว
- ~~player name ยังไม่ถูกใช้ครบ flow~~ ✅ แก้แล้ว
- market sync ยังไม่ตรง state จริงทุกจุด ⏳

ก่อนจะเพิ่มโหมดใหม่ ควรทำให้ระบบผู้เล่นและ state sync นิ่งก่อน

## 2. Product Goal for the Next Build

เป้าหมายของ build ถัดไปควรเป็น:

**ทำให้เกมรอบแรกเล่นสนุกและมี teamwork ชัด โดยไม่เพิ่ม complexity เกินจำเป็น**

สิ่งที่ build นี้ต้องได้:

1. join / lobby / start / play / lose ทำงานลื่น
2. ผู้เล่นเข้าใจเป้าหมายของทีมภายในไม่กี่วินาที
3. แพ้แล้วอยากเล่นใหม่ทันที
4. feature ใหม่ต้องใช้จุดแข็งของเกมนี้ คือ shared screen + mobile controllers

## 3. Recommended Milestones

ลำดับแนะนำ:

1. `Milestone 0` Stabilize current prototype
2. `Milestone 1` Convoy / Escort Mode MVP
3. `Milestone 2` Game Over Recap + Fast Replay
4. `Milestone 3` Shared Reinforcement Mode
5. `Milestone 4` Role Draft Foundation
6. `Milestone 5` Command Relay Exploration

สิ่งสำคัญ:

- ทำทีละ milestone
- อย่าผสม `Milestone 1` กับ `Milestone 5`
- อย่าแตะ protocol ขนาดใหญ่ก่อน `Milestone 0` เสร็จ

## 4. Milestone 0 — Stabilize Current Prototype

เป้าหมาย:

- ทำให้ core loop ปัจจุบันน่าเชื่อถือ
- ลด state mismatch ระหว่าง game กับ mobile
- ปิดปัญหาพื้นฐานก่อนเพิ่ม objective mode

> `Flag (Codex - 2026-03-12):` ✅ `lobby polish` ใน milestone นี้เสร็จแล้ว
>
> scope:
> - เพิ่มสถานะ `connected / ready` ใน player state ฝั่ง host
> - ปรับ lobby list ให้เห็นชื่อและสถานะชัดขึ้น
> - ให้ host start game จากรายชื่อผู้เล่นที่ ready จริง
>
> files:
> - `src/game/state.rs`
> - `src/game/mod.rs`
> - `src/game/renderer.rs`

## 4.1 Functional Scope

ต้องทำ:

1. ~~explicit join event จาก server ไป game~~ ✅ `Flag (Claude - 2026-03-12)`: done — `PlayerInput.name` + `drain_inputs`
2. ~~explicit disconnect event จาก server ไป game~~ ✅ `Flag (Claude - 2026-03-12)`: done — `PlayerInput.disconnect` + slot freed
3. ~~ใช้ชื่อ player จริงแทนชื่อ default~~ ✅ `Flag (Claude - 2026-03-12)`: done — `p.name` set from Join message
4. sync player state ระหว่าง `Market` ⏳ Task Card B1
5. ~~ปรับ lobby ให้เห็น player พร้อมชื่อและ readiness ชัด~~ ✅ `Flag (Codex - 2026-03-12)`

ไม่ต้องทำใน milestone นี้:

- role system
- objective ใหม่
- revive
- mode selector แบบเต็ม

## 4.2 Proposed Data Model Changes

ใน [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs):

- เพิ่ม game-facing events สำหรับ lifecycle
- แยก `PlayerInput` ออกจาก lifecycle events

โครงสร้างที่แนะนำ:

```rust
pub enum GameCommand {
    Join { player_id: u8, name: String },
    Disconnect { player_id: u8 },
    Input { player_id: u8, joy: [f32; 2], fire: bool },
    Buy { player_id: u8, item: String },
}
```

ถ้าไม่อยาก rename ใหญ่:

- คง `PlayerInput` ไว้
- เพิ่ม enum ใหม่เช่น `GameCommand`

ใน [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs):

- `Player` ต้องเก็บ `name` จาก join flow จริง
- เพิ่ม field เช่น:
  - `connected: bool`
  - `ready: bool` หรือ derive จาก connect state

ไม่จำเป็นต้องมี reconnect support เต็มรูปแบบในรอบนี้

## 4.3 Files Expected to Change

ไฟล์หลัก:

- [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- [`src/server/ws_handler.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/ws_handler.rs)
- [`src/server/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/mod.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)
- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

## 4.4 Acceptance Criteria

- ~~เมื่อผู้เล่นกด join แล้ว player ต้องถูกสร้างทันที แม้ยังไม่ขยับ joystick~~ ✅ `Flag (Claude - 2026-03-12)`
- ~~ชื่อบนจอใหญ่ต้องเป็นชื่อจากมือถือ~~ ✅ `Flag (Claude - 2026-03-12)`
- ~~เมื่อปิด browser หรือ socket หลุด player ต้องถูก remove หรือ mark disconnected อย่างถูกต้อง~~ ✅ `Flag (Claude - 2026-03-12)`
- ช่วง market ค่า `coins`, `hp`, `weapon_level` บนมือถืออัปเดตทันทีหลังซื้อ ⏳ Task Card B1
- ~~host สามารถเริ่มเกมได้จากรายชื่อผู้เล่นที่ join จริง~~ ✅ `Flag (Codex - 2026-03-12)`

## 4.5 Verification

manual test:

1. เปิดมือถือ 2-3 เครื่อง
2. join โดยยังไม่แตะจอย
3. เช็กว่าจอ host เห็นชื่อครบ
4. ปิด 1 เครื่อง
5. เช็กว่า player count ลดลง
6. เข้า market แล้วซื้อของ
7. เช็กว่ามือถือเห็น coins และ affordability เปลี่ยนทันที

## 5. Milestone 1 — Convoy / Escort Mode MVP

> `Flag (Antigravity - 2026-03-12, IN PROGRESS):` milestone นี้ถูกหยิบไป implement แล้วในฝั่ง Antigravity
>
> scope ที่ทำ:
> - เพิ่ม ConvoyCore และ GameMode
> - วาด core และ HP bar
> - ตรวจสอบ collision ของ core กับศัตรู
> - เพิ่ม condition Game Over เมื่อ core แตก

นี่คือ milestone แรกที่ควรเพิ่ม teamwork ใหม่จริง

เป้าหมาย:

- เพิ่ม shared team objective
- ทำให้ทีมมีเหตุผลต้องช่วยกันมากกว่าการยิงเพื่อ score
- ไม่ขยาย mobile protocol มากเกินไป

## 5.1 Product Definition

ในโหมดนี้ ทีมต้องคุ้มกัน `Convoy Core` ให้รอดจนจบ wave หรือจน progress ถึงเป้าหมาย

fail conditions:

- ผู้เล่นทุกคนตาย
- หรือ `Convoy Core` HP เป็น 0

win loop:

- ปกป้อง core
- เคลียร์ศัตรู
- ผ่าน wave
- เข้า market
- เล่น wave ถัดไป

## 5.2 MVP Rules

กติกา MVP:

- Core อยู่กึ่งกลางล่างจอ
- มี HP ของตัวเอง
- core ค่อยๆ ขยับไปข้างหน้าเมื่อ wave ยังเดินอยู่ หรือคงที่ถ้าต้องการเวอร์ชันง่ายกว่า
- enemy บางส่วน target core แทนผู้เล่น
- ถ้า core ตาย -> `GameOver`
- market ยังทำงานเหมือนเดิม
- ไม่มีระบบซ่อม core ใน MVP

เวอร์ชันง่ายสุดที่แนะนำ:

- `core stationary`
- ศัตรูบางชนิดชน core แล้วสร้าง damage
- ผู้เล่นมีหน้าที่ปกป้องพื้นที่รอบ core

เหตุผล:

- implementation ง่ายกว่า moving payload
- teamwork impact ยังสูงอยู่

## 5.3 Proposed Data Model

ใน [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs) เพิ่ม:

```rust
pub enum GameMode {
    Classic,
    Convoy,
}

pub struct ConvoyCore {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub radius: f32,
    pub alive: bool,
}
```

ใน `GameState` เพิ่ม:

- `mode: GameMode`
- `convoy_core: Option<ConvoyCore>`

ถ้าต้องการให้ง่ายและ explicit:

- เพิ่ม helper เช่น `is_convoy_mode()`

## 5.4 Game Loop Integration

ใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs):

ต้องเพิ่ม:

1. spawn core ตอนเริ่มเกมหรือเริ่ม wave
2. update core state
3. ตรวจ collision ระหว่าง enemy กับ core
4. ตรวจ fail condition ของ core
5. broadcast event เมื่อ core โดนตีหนักหรือใกล้แตก

flow ที่แนะนำ:

- ใน `update_lobby`:
  - ถ้า mode เป็น `Convoy` และเริ่มเกม ให้สร้าง core

- ใน `Phase::Playing`:
  - เรียก `update_convoy_core()`
  - เรียก `resolve_core_collisions()`
  - เรียก `check_convoy_fail()`

- ใน `reset()`:
  - reset core ด้วย

## 5.5 Enemy Behavior for MVP

ไม่ควร rewrite enemy AI ทั้งระบบ

วิธีง่าย:

- enemy ยังเคลื่อนแบบเดิม
- ถ้าศัตรูลงมาถึง zone ของ core หรือชน circle ของ core -> ทำ damage แล้วหายไป

วิธีนี้พอสำหรับ MVP เพราะ:

- ไม่ต้องมี steering AI
- ยังสร้างความรู้สึกว่าทีมกำลังกัน objective

optional improvement:

- เพิ่ม enemy variant ที่ target core โดยเฉพาะใน milestone ถัดไป

## 5.6 Rendering Requirements

ใน [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs):

ต้องเพิ่ม:

- การวาด core
- core HP bar
- objective label เช่น `DEFEND THE CORE`
- ถ้า hp ต่ำ ให้มี warning visual

HUD ขั้นต่ำที่ต้องมี:

- มุมบนจอหรือกลางบน: `Core HP`
- label ของ mode
- event text เมื่อ core โดนตี

## 5.7 Mode Selection

สำหรับ MVP ไม่ต้องทำเมนูใหญ่

ตัวเลือกง่าย:

- กด key บน host เช่น `[M]` สลับ `Classic / Convoy`
- lobby แสดง mode ปัจจุบัน

ถ้าจะให้มือถือเลือก mode ในภายหลัง ค่อยทำทีหลัง

## 5.8 Files Expected to Change

- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)

อาจแตะ:

- [`SUMMARY.md`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/SUMMARY.md)

## 5.9 Acceptance Criteria

- host เลือก `Convoy` mode ได้จาก lobby
- เริ่มเกมแล้ว core ปรากฏบนจอ
- enemy ชน core แล้ว core hp ลด
- core hp หมดแล้วเกมจบ
- ถ้า core ยังอยู่และผ่าน wave ได้ market ต้องยังทำงานเหมือนเดิม
- gameplay ต้องไม่พังใน `Classic` mode

## 5.10 Verification

manual test:

1. start classic mode
2. เช็กว่า behavior เดิมยังไม่พัง
3. restart
4. switch to convoy mode
5. ปล่อยศัตรูชน core
6. เช็กว่า hp ลดและ game over เมื่อถึง 0
7. เล่นจนจบ wave
8. เช็กว่า market phase ยังมา

## 6. Milestone 2 — Game Over Recap + Fast Replay

> `Flag (Codex - 2026-03-12):` ✅ milestone นี้ถูก implement แล้วในฝั่ง Codex โดยโฟกัสที่ host-side game state / loop / renderer
>
> scope ที่ทำ:
> - เพิ่ม run recap summary ตอน `GameOver`
> - เพิ่ม `quick replay` จากหน้า game over
> - เพิ่ม `return to lobby` โดย preserve roster เดิมระหว่าง restart run
> - ปรับ recap layout ให้รองรับ player หลายคนชัดขึ้น
> - เพิ่ม convoy-specific summary (`mode` และ `core HP`)
> - เพิ่ม `Press [M] to change mode` จากหน้า game over
>
> ไฟล์ที่แตะ:
> - `src/game/state.rs`
> - `src/game/mod.rs`
> - `src/game/renderer.rs`
> - `CODEX_FLAG_MILESTONE2_GAMEOVER_RECAP.md`

เป้าหมาย:

- ลด friction หลังแพ้
- ให้ผู้เล่นเห็นผลลัพธ์ของ run
- กระตุ้นให้กดเล่นอีกรอบทันที

## 6.1 Product Definition

เมื่อเกมแพ้:

- ไม่ขึ้นแค่ `GAME OVER`
- แต่แสดง recap สั้นๆ:
  - ถึง wave ไหน
  - ใคร score สูงสุด
  - ใครเก็บ coins สูงสุด
  - ถ้าเป็น convoy mode ให้แสดง core survived ถึงไหน

แล้วให้กด:

- `R` เพื่อ restart run เดิม
- หรือ `Space` เพื่อ quick replay

## 6.2 Proposed Data Model

ใน [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs) เพิ่ม:

```rust
pub struct RunSummary {
    pub final_wave: u32,
    pub top_score_player: Option<u8>,
    pub top_coins_player: Option<u8>,
    pub mode: GameMode,
    pub convoy_core_remaining_hp: Option<i32>,
}
```

ใน `GameState` เพิ่ม:

- `last_summary: Option<RunSummary>`

## 6.3 Logic Changes

ใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs):

- ตอน `check_game_over()` ให้สร้าง summary ก่อนเปลี่ยน phase
- ตอน `reset()` ให้รองรับ quick replay แบบไม่ต้องทิ้ง mode ที่เลือก

ถ้าต้องการ UX ที่ดี:

- แยก `full reset` กับ `restart_run_preserve_mode`

## 6.4 Rendering Changes

ใน [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs):

- ทำ game over panel ใหม่
- แสดง player highlights
- แสดง prompt ที่อ่านง่าย:
  - `Press R to restart`
  - `Press M to change mode`

## 6.5 Acceptance Criteria

- หลังแพ้ ต้องเห็น wave ที่ไปถึง
- ต้องมีอย่างน้อย 2 stats cards หรือ recap lines
- restart ต้องกลับไป lobby หรือ restart run ได้อย่างเร็วและไม่งง
- mode selection เดิมต้องยังอยู่ถ้าเลือก quick replay แบบ preserve mode

## 7. Milestone 3 — Shared Reinforcement Mode

milestone นี้เริ่มเพิ่ม team dependency มากขึ้น
ควรทำหลัง `Milestone 1` และ `Milestone 2` เสร็จแล้ว

เป้าหมาย:

- ทำให้การตายมีผลต่อทั้งทีม
- เพิ่ม dramatic comeback

## 7.1 MVP Rules

- ทีมมี `reinforcements` ร่วมกัน เช่น 5
- เมื่อ player ตาย:
  - ถ้า reinforcement > 0 ให้ลด 1
  - ผู้เล่นเข้าสถานะ respawning
  - กลับมาเกิดใหม่หลัง cooldown
- ถ้า reinforcement = 0 และผู้เล่นทั้งหมดตาย -> game over

MVP นี้ยังไม่ต้องมี:

- revive by teammate
- revive beacon
- shared hp

## 7.2 Data Model

ใน `Player` เพิ่ม:

- `respawn_timer: f32`
- `is_respawning: bool`

ใน `GameState` เพิ่ม:

- `reinforcements: i32`
- `max_reinforcements: i32`

## 7.3 Logic Changes

ใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs):

- แยก `player destroyed` ออกจาก `player permanently dead`
- update respawn timers ใน `Phase::Playing`
- spawn player กลับจุดปลอดภัยเมื่อ timer หมด
- ปรับ `check_game_over()` ให้ดูทั้ง reinforcements และ alive players

## 7.4 Rendering and Mobile

ใน [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs):

- เพิ่ม reinforcement counter
- แสดง respawn countdown เหนือชื่อผู้เล่นหรือ HUD

ใน [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs):

- อาจเพิ่ม field ใน `State` เช่น:
  - `alive`
  - `respawning`
  - `respawn_seconds`

ใน [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html):

- แสดง overlay `RESPAWNING`

## 7.5 Acceptance Criteria

- player ตายแล้วไม่หายถาวรทันทีถ้ายังมี reinforcement
- reinforcement ลดตามจำนวนการตาย
- player กลับมาเกิดใหม่ได้
- game over เกิดเมื่อทีมหมด reinforcement และไม่มีใครอยู่ในสนาม

## 8. Milestone 4 — Role Draft Foundation

> `Flag (Codex - 2026-03-13):` ✅ **DONE**
>
> scope ที่ทำ:
> - เพิ่ม role foundation (`None`, `Vanguard`, `Guardian`, `Salvager`) ใน player state
> - ให้ host เลือก role จากหน้า lobby ด้วย keyboard (`LEFT/RIGHT` เลือกผู้เล่น, `Q/E` เปลี่ยน role)
> - ทำ role effects จริงกับ gameplay: fire cooldown, max HP, invincibility window, pickup radius, coin bonus
> - แสดง role ใกล้ชื่อผู้เล่นบน host screen
>
> ไฟล์ที่แตะ:
> - `src/game/state.rs`
> - `src/game/mod.rs`
> - `src/game/renderer.rs`

นี่เป็น multiplier ของ market และ teamwork
แต่ไม่ควรทำก่อน objective mode

เป้าหมาย:

- ให้ผู้เล่นมี identity ต่างกันแบบง่าย
- ไม่ทำระบบ active abilities ก่อน

## 8.1 MVP Roles

แนะนำ 3 role ก่อน:

### Vanguard

- ยิงแรงขึ้นเล็กน้อย หรือ fire cooldown ดีขึ้นเล็กน้อย
- hp ต่ำลงเล็กน้อย

### Guardian

- max hp มากขึ้น 1
- invincibility หลังโดนตีเพิ่มเล็กน้อย

### Salvager

- เก็บ coin/drop ได้จากระยะไกลขึ้น
- ได้ coins ต่อ drop เพิ่มเล็กน้อย

## 8.2 Data Model

ใน `Player` เพิ่ม:

```rust
pub enum Role {
    None,
    Vanguard,
    Guardian,
    Salvager,
}
```

## 8.3 UI Recommendation

สำหรับ MVP:

- เลือก role จาก host screen ด้วย keyboard หรือรอบแรกตั้ง default ให้ทุกคน `None`
- ถ้าจะให้มือถือเลือก role ค่อยทำใน milestone ถัดไป

## 8.4 Acceptance Criteria

- role ต้องส่งผลกับ gameplay จริง
- role ต้องแสดงใกล้ชื่อผู้เล่น
- market และ score loop ต้องไม่พัง

## 9. Milestone 5 — Command Relay Exploration

นี่คือ differentiator ที่ดีที่สุด
แต่ไม่ควรเริ่มจนกว่า milestone ก่อนหน้าจะนิ่ง

เป้าหมาย:

- ใช้มือถือเป็น second screen
- เพิ่ม asymmetrical information

## 9.1 MVP Scope

MVP ที่เล็กที่สุด:

- มี `Commander` 1 คนต่อ wave
- commander เห็น warning message เฉพาะตัว
- commander มีปุ่ม ping:
  - left
  - mid
  - right
- ping แสดงบนจอใหญ่ 2 วินาที

ยังไม่ต้องมี:

- minimap
- special commander-only market
- scan mechanic ซับซ้อน

## 9.2 Protocol Changes

ใน [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs):

ต้องเพิ่ม:

- client message ใหม่ เช่น `ping`
- server message ใหม่ เช่น:
  - `role_assignment`
  - `commander_alert`

ตัวอย่าง:

```rust
pub enum ClientMsg {
    Join { name: String },
    Input { joy: [f32; 2], fire: bool },
    Buy { item: String },
    Ping { lane: PingLane },
}
```

และ:

```rust
pub enum ServerMsg {
    Joined { player_id: u8, slot: u8 },
    Full,
    State { ... },
    Event { msg: String },
    Market { items: Vec<MarketItem> },
    RoleAssignment { role: String },
    CommanderAlert { code: String, text: String },
}
```

## 9.3 Files Expected to Change

- [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- [`src/server/ws_handler.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/ws_handler.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)
- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

## 9.4 Risk

นี่คือ milestone ที่เสี่ยงสุดเพราะแตะ:

- protocol
- server fanout
- game state
- mobile UI
- UX clarity

ดังนั้นห้ามเปิดหลาย sub-features พร้อมกัน

## 10. Parallelization Guide for AI Agents

section นี้เขียนเพื่อใช้แตกงานไปหลาย agent โดยลด merge conflict

## 10.1 Safe Parallel Workstreams

### Workstream A — Server Protocol

ownership:

- [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- [`src/server/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/mod.rs)
- [`src/server/ws_handler.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/ws_handler.rs)

responsibility:

- game commands
- join/disconnect lifecycle
- targeted mobile messages

### Workstream B — Game State and Logic

ownership:

- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/enemy.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/enemy.rs)
- [`src/game/item.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/item.rs)

responsibility:

- mode state
- convoy core
- reinforcements
- run summary

### Workstream C — Rendering and HUD

ownership:

- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)

responsibility:

- lobby mode label
- convoy HUD
- recap screen
- reinforcement indicators

### Workstream D — Mobile UI

ownership:

- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

responsibility:

- state sync polish
- respawning overlay
- future role/commander UI

### Workstream E — Documentation and Validation

ownership:

- [`SUMMARY.md`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/SUMMARY.md)
- [`Plan.md`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/Plan.md)
- [GAME_JOURNEY.md](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/GAME_JOURNEY.md)
- [TEAMWORK_MODES.md](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/TEAMWORK_MODES.md)

responsibility:

- update docs after behavior changes
- keep summary consistent with code

## 10.2 Merge Order

แนะนำลำดับ merge:

1. server lifecycle
2. game state logic
3. renderer/HUD
4. mobile UI
5. docs

เหตุผล:

- server + game state เป็นของจริงที่ระบบอื่นพึ่ง
- renderer และ mobile เป็น consumer ของ state

## 11. Task Cards for AI Delegation

ด้านล่างคือ task format ที่สามารถ copy ไปให้ AI ตัวอื่นทำได้

## Task Card A1 — Explicit Join / Disconnect Lifecycle

> `Flag (Claude - 2026-03-12):` ✅ **DONE**
>
> approach ที่ใช้จริง: เพิ่ม `name: Option<String>` และ `disconnect: bool` เข้า `PlayerInput` (แทนการสร้าง `GameCommand` enum ใหม่) เพื่อลด scope และ compile complexity
>
> ไฟล์ที่แตะ:
> - `src/server/messages.rs` — เพิ่ม 2 fields ใน `PlayerInput`
> - `src/server/ws_handler.rs` — forward Join name + clone input_tx_dc สำหรับ disconnect notification
> - `src/server/mod.rs` — decrement `player_count` Arc หลัง `handle_socket` return
> - `src/game/mod.rs` — `drain_inputs` handle disconnect (remove + Lobby fallback) และ name (set p.name + broadcast)

เป้าหมาย:

- ทำให้ player join/disconnect เป็น explicit command

scope:

- server messages
- ws handler
- game command ingestion

files:

- [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- [`src/server/ws_handler.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/ws_handler.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)

definition of done:

- ~~join สร้าง player ทันที~~ ✅
- ~~disconnect remove player หรือ mark disconnected~~ ✅
- ~~player names มาจากมือถือจริง~~ ✅

## Task Card B1 — Market Sync Fix

> `Flag (Antigravity - 2026-03-12):` ✅ server-side done — `broadcast_player_states` เพิ่มใน `Phase::Market` loop, `PlayerState` + `SendMarket` ส่งทันทีหลัง purchase ใน `drain_inputs`
>
> `Follow-up Flag (Claude - 2026-03-12):` ✅ **FULLY DONE** — mobile-side complete
>
> approach ที่ใช้จริง: แทน `renderMarketItems()` full-rebuild ทุก frame ด้วย `refreshMarketAffordability()` in-place update ที่:
> - อัปเดต `#market-coins` display ทันที
> - toggle `.can-afford` border และ `disabled` ของแต่ละ button ตาม `myCoins` ล่าสุด
> - skip button ที่ถูก disable ไปแล้ว (pending purchase) — ป้องกัน double-buy
> - เพิ่ม `data-cost` attribute บน button เพื่อ parse ราคาโดยไม่ต้อง parse text

เป้าหมาย:

- ทำให้มือถือเห็น state ล่าสุดระหว่าง market

scope:

- broadcast player state ระหว่าง `Phase::Market`
- update affordability UI บนมือถือ

files:

- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

definition of done:

- ซื้อแล้ว coins/update ทันที

## Task Card B2 — Convoy Core MVP Logic

> `Flag (Antigravity - 2026-03-12):` ✅ **DONE**

เป้าหมาย:

- เพิ่ม convoy core และ fail condition

scope:

- state structs
- update logic
- core collision

files:

- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)

definition of done:

- convoy mode เล่นได้
- core ตายแล้ว game over
- classic mode ไม่พัง

## Task Card C1 — Convoy HUD

> `Flag (Antigravity - 2026-03-12):` ✅ **DONE**

เป้าหมาย:

- แสดง core, hp bar, objective label

scope:

- renderer only

files:

- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)

dependency:

- ต้องรู้ data shape ของ convoy core ก่อน

definition of done:

- เห็น objective ชัดบนจอใหญ่

## Task Card C2 — Game Over Recap

> `Flag (Codex - 2026-03-12):` ✅ **DONE**
>
> approach: เพิ่ม `RunSummary` struct ใน `state.rs`, `build_run_summary()` + `quick_replay()` + `restart_to_lobby()` ใน game loop (`mod.rs`), `draw_game_over()` ใน `renderer.rs` แสดง 3 summary cards (Top Score, Coin Leader, Crew Status) + pilot recap table + `[Space] Quick Replay` / `[R] Return to Lobby`
>
> ไฟล์ที่แตะ:
> - `src/game/state.rs` — `RunSummary`, `quick_replay()`, `restart_to_lobby()`
> - `src/game/mod.rs` — `check_game_over()` เรียก `build_run_summary()`, GameOver phase key handlers
> - `src/game/renderer.rs` — `draw_game_over()` recap panel

เป้าหมาย:

- ทำ game over panel ใหม่พร้อม summary สั้นๆ

scope:

- render recap
- consume summary state

files:

- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)

definition of done:

- แพ้แล้วมี recap และ restart flow ชัด

## Task Card D1 — Respawn Overlay

> `Flag (Antigravity - 2026-03-12):` ✅ **DONE**

เป้าหมาย:

- เตรียมมือถือให้รองรับ reinforcement mode ภายหลัง

scope:

- mobile state fields
- respawning overlay

files:

- [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

definition of done:

- มือถือแสดงสถานะ respawning ได้ถ้ามี field เข้ามา

## 12. Suggested Delivery Strategy

ถ้ามี AI หลายตัวทำงานพร้อมกัน แนะนำแบบนี้:

### Batch 1

- ~~A1 Explicit Join / Disconnect Lifecycle~~ ✅ `Flag (Claude - 2026-03-12)`: done
- ~~B1 Market Sync Fix~~ ✅ `Flag (Antigravity - 2026-03-12)`: done

### Batch 2

- ~~B2 Convoy Core MVP Logic~~ ✅ `Flag (Antigravity - 2026-03-12)`: done
- ~~C1 Convoy HUD~~ ✅ `Flag (Antigravity - 2026-03-12)`: done

### Batch 3

- ~~C2 Game Over Recap~~ ✅ `Flag (Codex - 2026-03-12)`: done

### Batch 4

- ~~D1 Respawn Overlay prep~~ ✅ `Flag (Antigravity - 2026-03-12)`: done
- เริ่ม Shared Reinforcement หลัง Batch 3 stable

ห้ามทำพร้อมกัน:

- `Command Relay` กับ `Join/Disconnect refactor`
- `Role Draft` กับ `Shared Reinforcement`

เหตุผล:

- ทั้งคู่แตะ `Player` shape และ mobile protocol พร้อมกัน มีโอกาสชนกันสูง

## 13. Definition of Success for This Document

เอกสารนี้ถือว่าใช้ได้ ถ้า AI agent อ่านแล้วสามารถตอบต่อได้ชัดว่า:

1. milestone ถัดไปคืออะไร
2. ต้องแก้ไฟล์ไหน
3. อะไรคือ non-goal
4. เทสอะไรบ้าง
5. งานไหนแยกขนานกันได้

## 14. Immediate Recommendation

ถ้าจะเริ่มลงมือจริงตอนนี้ ให้เริ่มตามลำดับนี้:

1. `Milestone 0`
2. `Milestone 1`
3. `Milestone 2`

อย่าเพิ่งเริ่ม `Milestone 5`

ถ้าทีม AI จะรับงานต่อ:

- agent แรกควรทำ lifecycle + market sync
- agent ที่สองควรทำ convoy core logic
- agent ที่สามควรทำ renderer / recap screen

หลังจาก 3 ส่วนนี้เสร็จแล้ว ค่อยตัดสินใจว่าจะไป `Shared Reinforcement` หรือหยุด playtest ก่อน
