# Starship Space — Implementation Plan

เอกสารนี้สรุปประเด็นสำคัญที่พบจากการวิเคราะห์ codebase ปัจจุบัน และลำดับงานที่ควรทำต่อเพื่อให้เกมพร้อมใช้งานมากขึ้น

## Current Assessment

สถาปัตยกรรมหลักใช้ได้ดี:
- `macroquad` รันบน main thread
- Axum WebSocket server รันบน background thread
- game กับ server คุยกันผ่าน Tokio channels

อย่างไรก็ตาม มีประเด็น behavior ที่ควรแก้ก่อนงาน polish เพราะกระทบ flow การเล่นจริง

## Priority Findings

### 1. Player disconnect ยังไม่ถูกจัดการ

ปัญหา:
- server เพิ่ม `player_count` ตอน connect แต่ไม่ลดตอน disconnect
- game ไม่มี event สำหรับ remove player
- slot ผู้เล่นจึงรั่ว และ server อาจเต็มถาวรหลังมีการเข้าออกหลายครั้ง

ผลกระทบ:
- ผู้เล่นใหม่เข้าไม่ได้หลังครบ 8 การเชื่อมต่อสะสม
- ผู้เล่นที่หลุดยังค้างใน game state

## 2. Join flow ยังไม่ complete จริง

ปัญหา:
- server ส่ง `joined` ทันทีที่เปิด socket
- แต่ game จะสร้าง player ก็ต่อเมื่อมี `input` หรือ `buy`
- ถ้ากด Join แล้วไม่แตะจอย ผู้เล่นยังไม่ถูกนับใน lobby

ผลกระทบ:
- host อาจเริ่มเกมไม่ได้ แม้ controller ขึ้นว่า join แล้ว
- ความหมายของ `join` ระหว่าง client/server/game ไม่ตรงกัน

## 3. Player name จากมือถือยังไม่ถูกใช้

ปัญหา:
- `Join { name }` ถูก log อย่างเดียว
- game ยังใช้ชื่อ default เช่น `P1`, `P2`

ผลกระทบ:
- UI และ lobby ไม่สะท้อนชื่อผู้เล่นจริง

## 4. Market UI sync ยังไม่ครบ

ปัญหา:
- ระหว่าง `Phase::Market` ไม่มีการ broadcast `PlayerState`
- หลังซื้อ item จะส่งแค่ market offers ใหม่
- client ใช้ `myCoins` เก่าคำนวณ affordability

ผลกระทบ:
- เหรียญบนมือถืออาจไม่อัปเดตทันที
- ปุ่มซื้ออาจ enable/disable ไม่ตรง state จริง

## 5. Summary documentation เริ่มไม่ตรงกับโค้ดจริง

ตัวอย่าง:
- weapon pattern ในโค้ดตอนนี้คือ `Lv1 = 1`, `Lv2 = 3`, `Lv3+ = 5`
- item drop rate ในโค้ดรวมเป็น 40%
- wave queue spawn จริงใช้ `pop()` จากท้ายลิสต์

ผลกระทบ:
- คนอ่านเอกสารอาจเข้าใจ behavior ผิดจากของจริง

## Recommended Execution Order

### Phase A: Fix player lifecycle

เป้าหมาย:
- ให้ `join` สร้าง player ทันที
- เก็บและใช้ชื่อผู้เล่นจริง
- ลด `player_count` ตอน disconnect
- ส่ง event เข้า game เพื่อ remove หรือ mark player หลุด

ผลลัพธ์ที่คาดหวัง:
- lobby count ถูกต้อง
- slot ไม่รั่ว
- reconnect behavior เริ่ม predictable

## Phase B: Fix market state synchronization

เป้าหมาย:
- ส่ง `PlayerState` ระหว่าง Market ด้วย
- refresh coins, hp, weapon level หลังซื้อ
- ทำให้ client render affordability จาก state ล่าสุด

ผลลัพธ์ที่คาดหวัง:
- controller UI ตรงกับ game state ตลอดช่วง market

## Phase C: Reconcile docs with implementation

เป้าหมาย:
- อัปเดต `SUMMARY.md` ให้ตรงกับ behavior จริง
- ถ้าต้องการ behavior ตามเอกสารเดิม ให้แก้โค้ดแทนและระบุให้ชัด

ผลลัพธ์ที่คาดหวัง:
- เอกสารใช้เป็น source of truth ได้อีกครั้ง

## Phase D: Next feature pass

หลังจาก behavior หลักนิ่งแล้ว ค่อยทำ:
- reconnect handling ที่สมบูรณ์
- sound effects
- visual polish
- high score / persistence

## Suggested First Implementation Scope

ถ้าจะเริ่มลงมือทันที ควรทำชุดนี้ก่อน:
1. เพิ่ม explicit join event จาก server ไป game
2. เพิ่ม explicit disconnect event จาก server ไป game
3. เก็บ player name ใน game state
4. broadcast player state ระหว่าง Market
5. อัปเดต `SUMMARY.md` หลัง behavior ใหม่เสถียรแล้ว
