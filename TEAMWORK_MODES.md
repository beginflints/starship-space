# Starship Space — Teamwork Modes Research, Spec, and Backlog

เอกสารนี้ตอบ 2 เรื่อง:

1. โหมดอะไรบ้างที่ทำให้การเล่นเป็นทีมสนุกขึ้นจริง
2. สำหรับ codebase ปัจจุบัน โหมดไหนควรทำก่อนและทำอย่างไร

อิงจากระบบในโปรเจคตอนนี้:

- phase หลักอยู่ใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- player state อยู่ใน [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- mobile protocol อยู่ใน [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
- WebSocket handling อยู่ใน [`src/server/ws_handler.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/ws_handler.rs)
- renderer/HUD อยู่ใน [`src/game/renderer.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/renderer.rs)

## 1. Research Lens

ผมไม่ได้จำกัดแค่ space shooter เพราะปัญหาที่ต้องแก้จริงคือ:

- จะทำให้ผู้เล่น "พึ่งพากัน" มากขึ้นได้อย่างไร
- จะทำให้มือถือเป็นมากกว่าจอยได้อย่างไร
- จะทำให้ความวุ่นวายแปลเป็น teamwork ไม่ใช่มั่ว

แนวคิดที่หยิบมาจากเกมอ้างอิง:

- `Overcooked`: co-op สนุกเพราะ objective บังคับการส่งไม้ต่อและการแบ่งหน้าที่
- `Keep Talking and Nobody Explodes`: teamwork เกิดแรงที่สุดเมื่อข้อมูลไม่เท่ากัน
- `Operation: Tango`: asymmetrical roles ทำให้การสื่อสารคือ core mechanic
- `Deep Rock Galactic`: mission objective + class identity + modifiers ทำให้ทีมมีโครง
- `Helldivers 2`: shared reinforcement pool และ mission pressure ทำให้ทุกความผิดพลาดมีผลต่อทั้งทีม
- `Among Us 3D` limited modes: role/objective พิเศษช่วยรีเฟรช core game โดยไม่ต้องเปลี่ยนเกมหลักทั้งก้อน

## 2. Constraints ของ Codebase นี้

ก่อนเลือกโหมด ต้องยอมรับข้อเท็จจริงของโปรเจคปัจจุบัน:

### สิ่งที่มีอยู่แล้ว

- wave shooter loop ชัดใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- market กับ item economy พื้นฐานมีอยู่แล้ว
- mobile controller ใช้งานได้แล้ว
- renderer แยก phase ชัดพอให้เพิ่ม HUD/overlay ได้

### สิ่งที่ยังไม่มี

- objective entity กลางทีม เช่น payload/core/beacon
- revive หรือ shared life system
- role/asymmetry system
- protocol สำหรับส่งข้อมูลเฉพาะบทบาทไปมือถือ
- mode selection / mutator framework

### ผลกระทบเชิงเทคนิค

ตอนนี้มือถือส่งได้แค่ `join`, `input`, `buy` ตาม [`src/server/messages.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/server/messages.rs)
และ server ส่งกลับได้แค่ `joined`, `state`, `event`, `market`

ดังนั้น:

- โหมดที่เพิ่ม objective บนจอใหญ่ทำง่ายกว่า
- โหมดที่ทำให้มือถือมีหน้าที่พิเศษจะสนุกมาก แต่ต้องเพิ่ม protocol และ UI

## 3. Evaluation Criteria

ผมใช้เกณฑ์ 4 ข้อนี้ประเมินทุกโหมด:

1. `Teamwork Impact`
   - บังคับให้ผู้เล่นพึ่งกันจริงไหม

2. `Clarity`
   - คนดูจอใหญ่เข้าใจ objective ใน 1-2 วินาทีไหม

3. `Implementation Fit`
   - ต่อเข้ากับ architecture ปัจจุบันได้แค่ไหน

4. `Replay Value`
   - เล่นหลายรอบแล้วยังไม่แบนไหม

## 4. Recommended Modes

## 4.1 Convoy / Escort Mode

### High Concept

ทีมต้องคุ้มกัน `Star Core` หรือ `Cargo Drone` ให้รอดตลอด route หรือจนกว่า progress bar จะเต็ม

### Why It Improves Teamwork

- ทีมมีสิ่งสำคัญร่วมกันที่ต้องปกป้อง
- ทุกคนต้องตัดสินใจว่าเมื่อไรควรไล่ฆ่า เมื่อไรควรกลับมาป้องกัน objective
- คนที่เล่นเก่งจะช่วยทีมได้โดยการ zoning และ cover ไม่ใช่แค่ทำ score

### Round Flow

1. เริ่ม wave พร้อม payload อยู่กลางล่างจอ
2. payload เคลื่อนไปข้างหน้าช้าๆ เมื่อพื้นที่ปลอดภัย
3. ศัตรูมีบางชุด target ผู้เล่น บางชุด target payload
4. ถ้า payload โดนจน HP หมด เกมแพ้ทันที
5. ผ่าน checkpoint แล้วเข้า market หรือ event room

### MVP Rule Set

- เพิ่ม objective entity ชื่อ `ConvoyCore`
- มี `hp`, `max_hp`, `x`, `y`, `progress`
- ศัตรูบางตัวบินตรงเข้าหา core
- ผู้เล่น heal core ไม่ได้ใน MVP
- market ซื้อ upgrade ให้ตัวเองตามปกติ

### Advanced Rule Set

- เพิ่มไอเท็มซ่อม core
- เพิ่ม shield windows จาก team action
- เพิ่ม split-lane threats ซ้าย/ขวา

### Controller Impact

แทบไม่ต้องเปลี่ยนมือถือในเฟสแรก

### HUD / UX Needs

- core HP bar ใหญ่บนจอ
- arrow หรือ ring highlight รอบ core
- event text เช่น `DEFEND THE CORE`

### Code Fit

เหมาะมากกับ codebase นี้ เพราะเป็นการเพิ่ม entity ใหม่ใน game loop
ไม่ต้องเปลี่ยน protocol มือถือก่อน

### Engineering Difficulty

`Medium`

เหตุผล:

- ต้องเพิ่ม objective state ใหม่
- ต้องเพิ่ม collision กับ fail condition
- แต่ไม่ต้องแก้ network contract เยอะ

### Recommendation

นี่คือโหมดแรกที่ควรทำถ้าอยากได้ teamwork impact สูงและเสี่ยงต่ำ

## 4.2 Command Relay Mode

### High Concept

ในแต่ละ wave จะมีผู้เล่น 1 คนเป็น `Commander`
มือถือของคนนี้เห็นข้อมูลพิเศษ เช่น:

- spawn direction
- weak point alert
- safe lane
- objective ping

แลกกับการที่ยานของเขายิงอ่อนลง หรือยิงไม่ได้ช่วงสั้นๆ

### Why It Improves Teamwork

- สร้างข้อมูลไม่สมมาตร
- บังคับให้คนหนึ่ง "พูด" และคนอื่น "เชื่อ/ตอบสนอง"
- ใช้จุดแข็งของโปรเจคนี้คือมือถือเป็น second screen

### Round Flow

1. ก่อนเริ่ม wave เกมสุ่มหรือให้เลือก commander
2. จอมือถือ commander แสดง tactical feed
3. commander ping หรือ callout ทิศอันตราย
4. ทีมเล่นตามข้อมูลที่คนอื่นไม่มี
5. จบ wave แล้ว role อาจหมุน

### MVP Rule Set

- commander เห็นแค่ `incoming_left / incoming_right / elite_warning`
- มีปุ่ม `PING LEFT`, `PING MID`, `PING RIGHT`
- ping ขึ้นบนจอหลักเป็น marker ใหญ่ 2 วินาที

### Advanced Rule Set

- commander เห็น weak point window ของ boss
- commander เลือก event decision ระหว่าง market
- commander ได้ minimap เฉพาะมือถือ

### Controller Impact

สูง

ต้องเพิ่ม:

- server message แบบ role-specific
- mobile UI เฉพาะ commander
- input ใหม่เช่น `ping`, `scan`, `mark`

### HUD / UX Needs

- commander badge บนจอหลัก
- ping visualization ชัดเจนมาก
- role handoff ระหว่าง wave

### Code Fit

สนุกมากเชิง product แต่แตะหลายชั้น:

- message protocol
- ws handler
- game event routing
- mobile controller screen

### Engineering Difficulty

`Medium-High`

### Recommendation

ควรเป็นโหมดที่ 2 หรือ 3 หลังจาก objective mode ตั้งหลักแล้ว

## 4.3 Shared Reinforcement Mode

### High Concept

ทีมมี `reinforcement pool` ร่วมกัน
เวลาผู้เล่นตาย จะไม่จบทันทีถ้ายังมี reinforcement เหลือ
ผู้เล่นคนนั้นจะ respawn ได้ตามกติกา เช่น:

- อัตโนมัติหลัง 5 วินาที
- หรือเมื่อทีมเก็บ beacon ได้

### Why It Improves Teamwork

- ทุกการตายส่งผลต่อทั้งทีม
- คนเก่งต้องเริ่มเล่นเพื่อทีม ไม่ใช่เพื่อตัวเอง
- ทำให้การช่วยกันเคลียร์ทางเพื่อรอเพื่อนกลับมามีความหมาย

### Round Flow

1. เริ่ม run พร้อม reinforcement เช่น 6
2. player ตาย -> reinforcement ลด 1
3. ผู้เล่น respawn ที่ safe zone หลัง delay
4. ถ้า reinforcement เหลือ 0 และทุกคนตาย -> game over

### MVP Rule Set

- ไม่มี revive mid-combat
- มีแต่ auto-respawn หลัง cooldown
- จำกัดให้ respawn ได้เฉพาะตอนศัตรูหนาแน่นไม่เกิน threshold หรือหลังช่วงสั้นๆ

### Advanced Rule Set

- add revive beacon
- add team pickup เพื่อเติม reinforcement
- add penalty เมื่อ respawn เช่น hp ไม่เต็ม

### Controller Impact

ต่ำ

มือถือแค่ต้องแสดงสถานะ `respawning...`

### HUD / UX Needs

- reinforcement counter ใหญ่บน HUD
- countdown เหนือชื่อผู้เล่นที่ตาย

### Code Fit

ปานกลาง

ระบบตายตอนนี้ใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs) เปลี่ยนผู้เล่นเป็น `alive = false` แล้วเข้าสู่ game over เมื่อทุกคนตาย
ดังนั้นต้องเพิ่ม life-cycle ใหม่ของ player แต่ไม่ต้องแตะ controller มาก

### Engineering Difficulty

`Medium`

### Recommendation

คุ้มมากถ้าทำคู่กับ Convoy เพราะจะเปลี่ยน mindset จาก solo survival เป็น team survival ทันที

## 4.4 Salvage Run Mode

### High Concept

ศัตรูจะดรอป `salvage crates` หรือ `energy cells`
ทีมต้องเก็บและส่งไปยัง `dock zone`
ครบ quota แล้วถึงจะจบ wave หรือเปิดประตูไป sector ถัดไป

### Why It Improves Teamwork

- บังคับให้ทีมคุ้มกันคนถือของ
- เพิ่มภารกิจแบบ "รับ-ส่ง" มากกว่ายิงหมด
- เปิดพื้นที่ให้คนเล่นหลายแบบในทีมเดียว

### Round Flow

1. wave เปิดด้วย quota เช่น `Deliver 8 Cells`
2. elite ตกของหรือมี crate spawn ตามจุด
3. คนถือของช้าลงและยิงไม่ได้ หรือยิงได้เบาลง
4. ทีมต้องคุมพื้นที่ให้คนส่งของถึง dock

### MVP Rule Set

- salvage เป็น pickup ชนิดใหม่
- เก็บแล้วตัวละครเคลื่อนช้าลง 25%
- ส่งของที่ zone กลางล่างจอ
- ครบ quota -> wave clear ทันที

### Advanced Rule Set

- crate ใหญ่ต้องมีผู้เล่น 2 คนอยู่ใกล้กัน
- enemies focus คนถือของ
- market มี upgrade สายขนส่ง

### Controller Impact

ต่ำถึงปานกลาง

ไม่จำเป็นต้องเพิ่ม input ใหม่ใน MVP

### HUD / UX Needs

- quota tracker
- carry icon เหนือหัวคนถือของ
- dropoff zone highlight

### Code Fit

ดี เพราะต่อจากระบบ item pickup เดิมได้
แต่ต้องเพิ่ม objective resolution มากกว่าไอเท็มธรรมดา

### Engineering Difficulty

`Medium`

### Recommendation

เหมาะมากเป็นโหมดลำดับ 2-4 หลังจากทีมคุ้น objective-based play แล้ว

## 4.5 Role Draft Mode

### High Concept

ก่อนเริ่ม run หรือก่อนเข้า sector ให้ผู้เล่นเลือก role คนละ 1 แบบ

ตัวอย่าง role:

- `Vanguard`: ดาเมจสูง, hp ต่ำ
- `Guardian`: มี pulse shield ต่อ wave
- `Salvager`: ดูด coin/drop ไกลขึ้น
- `Medic`: heal pickup แชร์ทีมเล็กน้อย
- `Scanner`: เห็น elite warning ก่อน

### Why It Improves Teamwork

- ผู้เล่นมีหน้าที่ที่ต่างกันชัด
- ทำให้ทีมคุยกันตั้งแต่ก่อนเริ่ม wave
- ทำให้ market และ drop มีเจ้าของเชิงกลยุทธ์มากขึ้น

### Round Flow

1. lobby หรือ market แสดง role choices
2. ทุกคน lock role
3. run นี้ใช้ perk ตาม role
4. market มีของบางชิ้นเสริม role

### MVP Rule Set

- มี 3 roles ก่อน: `Vanguard`, `Guardian`, `Salvager`
- role เป็น passive ล้วน ไม่มี active ability

### Advanced Rule Set

- role + active skill
- role-specific market items
- role synergy bonus

### Controller Impact

ปานกลาง

ต้องเพิ่มหน้าเลือก role บนมือถือหรือบน host screen

### HUD / UX Needs

- icon role ใกล้ชื่อผู้เล่น
- short tooltip ใน market

### Code Fit

ต้องขยาย `Player` state และ effect pipeline
แต่ยังไม่ต้องแก้ระบบ core loop มาก

### Engineering Difficulty

`Medium`

### Recommendation

ดีมากในฐานะ multiplier ของโหมดอื่น แต่ไม่ใช่โหมดแรกที่ควรทำเดี่ยวๆ

## 4.6 Weekly Ops / Mutator Mode

### High Concept

เพิ่ม preset rules หรือ mutators ให้ run เดิมมีรสใหม่

ตัวอย่าง:

- `One Core Only`
- `Shared HP`
- `No Market`
- `Coin Storm`
- `Fast Swarm`
- `Low Visibility`

### Why It Improves Teamwork

- บังคับให้ทีมเปลี่ยนแผน
- content cost ต่ำกว่าเพิ่มโหมดเต็ม
- ทำให้คนกลับมาเล่นรอบใหม่ด้วยเงื่อนไขต่างกัน

### MVP Rule Set

- มี mutator flags 3-5 ตัว
- เลือกก่อนเริ่มเกม
- แสดงบน lobby และ HUD

### Engineering Difficulty

`Low-Medium`

### Recommendation

ไม่ใช่โหมด teamwork ที่แรงที่สุด แต่เป็นระบบคูณมูลค่าของทุกโหมด

## 5. Priority Matrix

### Best First Build

1. `Convoy / Escort Mode`
2. `Shared Reinforcement Mode`
3. `Role Draft Mode`

เหตุผล:

- ทั้ง 3 ตัวเพิ่ม teamwork ชัด
- ยังใช้ controller ปัจจุบันได้เกือบหมด
- ต้นทุนต่ำกว่า Command Relay

### Best Product Differentiator

1. `Command Relay Mode`
2. `Convoy / Escort Mode`
3. `Salvage Run Mode`

เหตุผล:

- `Command Relay` ใช้จุดขายมือถือได้ชัดที่สุด
- ถ้าทำสำเร็จ เกมนี้จะไม่เหมือน shooter ทั่วไป

### Best Low-Risk Extension

1. `Weekly Ops / Mutator Mode`
2. `Shared Reinforcement Mode`
3. `Role Draft Mode`

## 6. Recommended Roadmap

## Phase 1: Team Objective Foundation

เป้าหมาย:

- ทำให้เกมมี objective กลางทีม
- เพิ่ม fail state ที่ไม่ใช่ทุกคนตายอย่างเดียว

งาน:

1. เพิ่ม `GameMode` enum
2. เพิ่ม `ObjectiveState` ใน game state
3. เพิ่ม HUD objective bar
4. เพิ่ม escort/convoy MVP

ผลลัพธ์:

- ได้โหมด teamwork จริงตัวแรก

## Phase 2: Team Survival Foundation

เป้าหมาย:

- ทำให้การตายมีผลต่อทั้งทีม

งาน:

1. เพิ่ม reinforcement pool
2. เพิ่ม respawn state ต่อ player
3. เพิ่ม HUD counter + respawn timer
4. ปรับ game over logic

ผลลัพธ์:

- run มี tension เชิงทีมมากขึ้นทันที

## Phase 3: Identity Foundation

เป้าหมาย:

- ทำให้ผู้เล่นแต่ละคนมีบทบาทต่างกัน

งาน:

1. เพิ่ม role enum ใน `Player`
2. เพิ่ม role selection UI
3. เพิ่ม passive effects 3 แบบ
4. ปรับ market ให้มี synergy กับ role

ผลลัพธ์:

- ทีมเริ่มมี composition และ metagame เบาๆ

## Phase 4: Asymmetric Controller Play

เป้าหมาย:

- ใช้มือถือให้เป็น second screen จริง

งาน:

1. เพิ่ม protocol สำหรับ role-specific data
2. เพิ่ม commander UI บนมือถือ
3. เพิ่ม ping/scan actions
4. เพิ่ม commander-assigned events

ผลลัพธ์:

- ได้ differentiator ที่แรงสุดของเกม

## Phase 5: Replay Layer

เป้าหมาย:

- ทำให้ run เดิมมี variation

งาน:

1. เพิ่ม mutator framework
2. ทำ preset weekly ops
3. เพิ่ม score modifiers / leaderboard later

## 7. Engineering Backlog by Scope

## Small

- เพิ่ม `GameMode` selector ใน lobby
- เพิ่ม HUD block สำหรับ objective/reinforcement
- เพิ่ม mutator flags พื้นฐาน
- เพิ่ม summary screen หลังจบเกม

## Medium

- objective entity ใหม่ เช่น convoy core
- shared reinforcement + respawn
- salvage pickup / dock zone
- role draft พร้อม passive 3 แบบ

## Large

- commander mobile UI
- role-specific server messages
- tactical ping and scan system
- multi-mode content with separate balance tables

## 8. Concrete Recommendation

ถ้าต้องเลือกแผนที่ "ถูกทั้ง product และ engineering" มากที่สุด:

### Milestone 1

ทำ `Convoy / Escort Mode`

เพราะ:

- teamwork impact สูง
- อธิบายง่าย
- ไม่ชนกับ network/mobile complexity มาก

### Milestone 2

ตามด้วย `Shared Reinforcement Mode`

เพราะ:

- ทำให้ objective mode ตึงขึ้นมาก
- ผู้เล่นเริ่มรู้สึกว่าทั้งทีมใช้ทรัพยากรร่วมกัน

### Milestone 3

ตามด้วย `Role Draft Mode`

เพราะ:

- เปิดทางให้ market สนุกขึ้น
- ทำให้ทีมคุยกันก่อนเริ่มแต่ละรอบ

### Milestone 4

ค่อยทำ `Command Relay Mode`

เพราะ:

- เป็นของเด่นเชิงเอกลักษณ์
- แต่ต้องแตะ protocol และ mobile UI หลายชั้น

## 9. If You Want One Flagship Mode

ถ้าคุณอยากมี "โหมดธง" ที่ใช้โปรโมตเกมนี้ได้จริง ผมเลือก:

`Command Relay Convoy`

สูตรคือ:

- ทีมคุ้มกัน core
- มี commander 1 คนต่อ wave
- commander เห็น threat forecast และ ping ได้
- reinforcement pool ใช้ร่วมกันทั้งทีม

นี่จะทำให้เกมนี้ไม่ใช่แค่ยิงยานหลายคน
แต่เป็นเกมที่มี:

- shared objective
- asymmetric communication
- social tension
- mobile identity

ซึ่งตรงกับจุดขายของโปรเจคนี้ที่สุด

## Sources

- Team17, `Overcooked! All You Can Eat`: https://www.team17.com/games/overcooked-all-you-can-eat/
- Team17, `Introducing Overcooked! 2`: https://www.team17.com/news/introducing-overcooked-2
- Steel Crate Games, `Keep Talking and Nobody Explodes`: https://keeptalkinggame.com/
- Steam, `Operation: Tango`: https://store.steampowered.com/app/1335790/Operation_Tango/
- Deep Rock Galactic official site: https://www.deeprockgalactic.com/
- PlayStation Blog, `Helldivers 2 hands-on report`: https://blog.playstation.com/2024/02/02/helldivers-2-hands-on-report-chaotic-co-op-and-empowering-stratagems/
- Innersloth, `Critical Cargo is Now Live`: https://www.innersloth.com/au3d-lte8-critical-cargo-launch-roadmap/
