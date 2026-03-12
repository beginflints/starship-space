# Codex Flag: Milestone 0 Lobby Polish

สถานะ:

- `DONE`

สิ่งที่กำลังทำ:

- เพิ่มสถานะ `connected` และ `ready` ใน player state ฝั่ง host
- mark `ready` เมื่อ server ส่ง `Join { name }` เข้า game สำเร็จ
- ปรับ lobby roster ให้เห็นชื่อ pilot กับสถานะชัดขึ้น
- gate การเริ่มเกมให้ใช้ผู้เล่นที่ `ready` จริง

ไฟล์ที่แตะ:

- `src/game/state.rs`
- `src/game/mod.rs`
- `src/game/renderer.rs`
- `IMPLEMENTATION_SPEC.md`
