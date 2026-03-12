# AGENTS.md

## Purpose

ไฟล์นี้ใช้เป็นกติกากลางสำหรับ AI agents และ dev ที่เข้ามาแก้ repo นี้ต่อ

## Working Rules

- ถ้ามีการเพิ่ม feature, ปรับ behavior, แก้ bug, หรือแก้ flow ที่ผู้เล่นสัมผัสได้ ให้ update `CHANGELOG.md` ใน patch เดียวกัน
- ถ้าเป็นงานที่ถูก claim จาก spec/task card ให้ update flag ใน `IMPLEMENTATION_SPEC.md` ด้วย
- ถ้าเริ่มทำงานที่มีโอกาสชนกับคนอื่น ให้ mark สถานะ `IN PROGRESS`
- ถ้าทำเสร็จและตรวจ compile/test ตามสมควรแล้ว ให้เปลี่ยนสถานะเป็น `DONE`

## Changelog Rules

- ใช้รูปแบบ `Keep a Changelog` แบบย่อ
- ให้เพิ่มรายการใหม่ไว้ใต้ `## [Unreleased]`
- ใช้หมวดต่อไปนี้ตามความเหมาะสม:
  - `Added`
  - `Changed`
  - `Fixed`
  - `Docs`
- เขียนแต่ละรายการให้สั้นและบอกผลลัพธ์ที่เกิดขึ้นกับเกมหรือ workflow

## Versioning Rules

- เริ่มใช้เวอร์ชันแบบ `0.x.y`
- `0.x.0` ใช้กับ milestone/feature set ที่ใหญ่พอสำหรับ playtest รอบใหม่
- `0.x.y` ใช้กับ patch, bug fix, polish, docs sync
- ถ้ายังไม่ได้ cut release ใหม่ ให้เก็บรายการไว้ใน `Unreleased`

## Commit Rules

- ทำ commit แบบเรียงลำดับทีละ logical change
- ข้อความ commit ให้สั้น ชัด และใช้ scope ที่อ่านแล้วเข้าใจทันที
- ถ้างานแตะทั้ง code และ changelog ให้รวมใน commit เดียวกัน
- ถ้างานยังทำไม่เสร็จ ไม่ควร commit แบบครึ่งๆ กลางๆ ยกเว้นผู้ใช้สั่งชัดเจน

## Recommended Flow

1. claim งานใน `IMPLEMENTATION_SPEC.md` ถ้างานมีโอกาสชน
2. ลงมือแก้ code
3. update `CHANGELOG.md`
4. ตรวจ `cargo check` หรือการ verify ที่เกี่ยวข้อง
5. เปลี่ยน flag เป็น `DONE`
6. commit เป็นลำดับ
