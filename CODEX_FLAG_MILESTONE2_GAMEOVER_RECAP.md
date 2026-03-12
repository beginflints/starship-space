# Codex Flag: Milestone 2

สถานะ:

- `DONE`

งานที่เลือก:

- `Milestone 2 - Game Over Recap + Fast Replay`

เหตุผลที่เลือก:

- เป็นก้อนที่แยกจาก `Milestone 0` และ `Milestone 1` ได้ค่อนข้างดี
- แตะเฉพาะ game loop, state, renderer เป็นหลัก
- ไม่ขยาย websocket protocol และไม่แก้ mobile controller
- ลดโอกาสชนกับงานอีก agent ที่อาจกำลังทำ player lifecycle, lobby sync หรือ objective mode

ไฟล์ที่แก้:

- `src/game/state.rs`
- `src/game/mod.rs`
- `src/game/renderer.rs`

สิ่งที่ทำ:

- เพิ่ม `RunSummary` เพื่อเก็บ recap ของรอบที่แพ้
- เก็บข้อมูล `Reached Wave`, `Top Score`, `Coin Leader`, และ `Crew Status`
- ปรับหน้า `GameOver` ให้เป็น recap panel แทนการแสดงผลแบบบรรทัดสั้นอย่างเดียว
- เพิ่ม `Pilot Recap` list เพื่อสรุป score/coins ของผู้เล่นแต่ละคน
- เพิ่ม `quick replay` ด้วยปุ่ม `Space`
- เพิ่ม `return to lobby` ด้วยปุ่ม `R`
- เพิ่ม `change mode` ด้วยปุ่ม `M` จากหน้า game over
- เพิ่ม convoy-specific summary (`mode` และ `core remaining HP`)
- ออกแบบให้ replay ใช้ roster เดิมและ reset เฉพาะ state ของ run โดยไม่ไปยุ่งกับชื่อผู้เล่น
- polish recap layout ให้รองรับ 6-8 คนได้อ่านง่ายขึ้น

หลักการออกแบบ:

- แยก `full reset` ออกจาก `run reset` เพื่อให้ replay เร็วขึ้น
- preserve roster และ player names เพื่อให้เล่นซ้ำทันทีโดย friction ต่ำ
- ใช้ summary ที่ snapshot ตอนแพ้จริง เพื่อไม่ให้ข้อมูล recap เปลี่ยนตาม frame ถัดไป
- จำกัด scope ให้อยู่ใน host-side gameplay/UI เพื่อ merge ง่าย
