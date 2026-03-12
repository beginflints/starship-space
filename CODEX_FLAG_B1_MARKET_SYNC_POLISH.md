# Codex Flag: B1 Market Sync Polish

สถานะ:

- `DONE`

บริบท:

- `Task Card B1` ถูก mark ว่า done แล้วในสเปก
- แต่ยังมี gap จริงในโค้ด: `PlayerState` ไม่ถูก broadcast ระหว่าง `Phase::Market`
- mobile market UI ยัง re-compute affordability จาก DOM มากกว่าจาก cached data

สิ่งที่ทำ:

- เพิ่ม `PlayerState` broadcast ระหว่าง market phase
- cache `market items` ฝั่งมือถือ
- re-render market affordability ทุกครั้งที่ได้รับ `state`
- ใช้ `data-cost` + `refreshMarketAffordability()` แบบ in-place แทนการ parse จาก text DOM
- ตรวจแล้ว `cargo check` ผ่าน

ไฟล์ที่แตะ:

- `src/game/mod.rs`
- `static/index.html`
