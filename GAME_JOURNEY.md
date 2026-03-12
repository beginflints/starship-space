# Starship Space — Research และ Game Journey

เอกสารนี้สรุป 2 ส่วน:

1. research เกมอ้างอิงที่ใกล้กับโปรเจคนี้
2. journey ของเกมฉบับที่เหมาะกับ codebase ปัจจุบัน

เป้าหมายคือไม่ออกแบบเกมลอยๆ แต่ต่อยอดจากของที่มีอยู่แล้วในโปรเจค:

- loop หลัก `Lobby -> Playing -> Market -> GameOver`
- local WiFi multiplayer ผ่านมือถือ
- ผู้เล่นแต่ละคนมี ship ของตัวเอง
- wave shooter + drops + market upgrades

อ้างอิงระบบปัจจุบันจาก:

- [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- [`src/game/market.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/market.rs)
- [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)

## 1. Research Summary

### A. `Lovers in a Dangerous Spacetime`

สิ่งที่น่าหยิบมาใช้:

- ภาพจำชัดมากว่าเป็น "เกมช่วยกันเอาตัวรอดในยานกลางความโกลาหล"
- จุดแข็งคือ simple controls แต่ teamwork ลึก
- shared-screen couch co-op ทำให้คนในห้องเดียวกันคุยกันตลอด

สิ่งที่ไม่ควรยกมาทั้งก้อน:

- เกมนั้นใช้เรือลำเดียวและแบ่ง station กัน แต่โปรเจคนี้วางฐานเป็นหลายยานคนละลำแล้ว
- ถ้าฝืนเปลี่ยนเป็น station-based จะกลายเป็นเปลี่ยน genre มากกว่า polish ของเดิม

ข้อสรุป:

- ควรยืม "energy ของความร่วมมือ" และ "เอกลักษณ์แบบ party co-op in space"
- แต่ให้ผู้เล่นยังคุมยานของตัวเองเหมือนเดิม

### B. `Sky Force Reloaded`

สิ่งที่น่าหยิบมาใช้:

- classic shooter + progression + collectibles
- ด่านสั้น กระชับ แต่มีเหตุผลให้เล่นต่อ
- local co-op เสริมความมัน ไม่ทำลาย readability ของเกมยิง

ข้อสรุป:

- โปรเจคนี้ควรเดินทางนี้: arcade shooter ที่เข้าถึงง่าย แต่มีแรงผลักจาก upgrade, coin, mission, boss และ build

### C. `Spaceteam`

สิ่งที่น่าหยิบมาใช้:

- มือถือเป็น controller ไม่ใช่ companion app
- ทุกคน join ง่าย ใช้อุปกรณ์ที่มีอยู่แล้ว
- social friction กลายเป็นความสนุก: เรียกกัน, เตือนกัน, ลุ้นพร้อมกัน

ข้อสรุป:

- มือถือของเกมนี้ควรรู้สึก "เข้าทันที เล่นได้ทันที" ไม่ใช่ UI ที่ต้องอ่านเยอะ
- ทุก journey ต้องลดเวลา setup และเพิ่มความชัดของ feedback บนมือถือ

### D. `Ship of Fools`

สิ่งที่น่าหยิบมาใช้:

- run มีเส้นทางความตึงเครียดชัด: fight -> repair/shop -> fight bigger -> boss
- co-op สนุกเพราะมีช่วงหายใจและช่วงตัดสินใจ ไม่ได้ยิงยาวๆ อย่างเดียว
- การเลือก build ทำให้แต่ละ run ต่างกัน

ข้อสรุป:

- โปรเจคนี้มี `Market` อยู่แล้ว ควรยกระดับให้เป็น "จังหวะตัดสินใจ" ไม่ใช่พัก 12 วินาทีเฉยๆ

## 2. Direction ที่เหมาะกับโปรเจคนี้

ตำแหน่งของเกมที่ควรเป็น:

**party-friendly co-op squad shooter**

ไม่ใช่ bullet hell สำหรับผู้เล่นฮาร์ดคอร์ล้วนๆ และไม่ใช่ roguelike ซับซ้อนเกินจำเป็น

ผู้เล่นควรรู้สึกว่า:

- เข้าเกมง่ายใน 30 วินาที
- ยิงสนุกตั้งแต่วินาทีแรก
- เล่นกับเพื่อนแล้วมีเรื่องให้คุยตลอด
- เก่งขึ้นจากการประสานงาน ไม่ใช่แค่เพราะเลเวลอาวุธสูง
- ตายแล้วอยากกดเล่นอีกรอบทันที

## 3. Design Pillars

### 1. Join Fast

สแกน QR > ใส่ชื่อ > เข้า controller > รอพร้อมเล่น

ช่วงก่อนเริ่มต้องเร็วมาก และไม่ทำให้ host ต้องอธิบายเยอะ

### 2. Read Fast

จอใหญ่ต้องอ่านง่ายใน 1 วินาที:

- ตอนนี้อยู่ phase ไหน
- wave ไหน
- ใครใกล้ตาย
- เป้าหมายตอนนี้คืออะไร

### 3. Cooperate Loud

เกมต้องกระตุ้นให้คนเรียกกันจริง:

- "เก็บฮีลตรงกลาง"
- "หลบซ้าย"
- "ช่วยเคลียร์แท็งก์ขวา"
- "อย่าพึ่งซื้อฮีล ผมจะเก็บกล่องให้"

### 4. Upgrade With Intent

market ต้องไม่ใช่เมนูธรรมดา แต่เป็นช่วงสร้างบทบาท:

- คนหนึ่งเป็น glass cannon
- คนหนึ่งเป็น tank/support
- คนหนึ่งเป็น coin farmer

### 5. Reset Clean

แพ้แล้วกลับมาเริ่มใหม่ได้ไว

เกมนี้เหมาะกับหลายรอบสั้นๆ ในงานปาร์ตี้หรือวงเพื่อน มากกว่ารอบเดียว 90 นาที

## 4. Player Journey ทั้งหมด

## 4.1 Before The Match

เป้าหมาย:

- ทำให้คนในห้องรู้ทันทีว่าเล่นยังไง
- ลด awkward time ก่อนเริ่ม

ประสบการณ์ที่ควรเกิด:

1. จอใหญ่ขึ้น title + QR + คำสั้นๆ เช่น `Scan to Join`
2. มือถือเปิดแล้วเจอหน้า join ที่มีชื่อเกมชัด
3. กรอกชื่อและเข้า slot ทันที
4. จอใหญ่แสดงรายชื่อ/สี/ตำแหน่งผู้เล่นที่ join แล้ว
5. ทุกคนลองขยับ joystick ได้ใน lobby เพื่อทดสอบ input

ข้อเสนอสำหรับโปรเจคนี้:

- ใน `Lobby` อย่าให้เป็นจอรอเฉยๆ
- ให้มี "warm-up zone" เช่น drone เป้าซ้อมเล็กๆ ลอยอยู่ด้านบน หรือ cursor test
- ทุกคนที่ขยับจอยครั้งแรกควรได้ feedback ทันทีบนจอใหญ่

เหตุผล:

- ตัดปัญหา "มือถือเชื่อมแล้วหรือยัง"
- เปลี่ยนเวลารอเพื่อน join ให้เป็นความสนุกเบาๆ

## 4.2 Onboarding 60 วินาทีแรก

แทนที่จะมี tutorial แยก ควรใช้ onboarding แบบเล่นจริงเลย

โครงสร้าง:

1. Wave 1 = ศัตรูน้อย เคลื่อนที่ตรง
2. notification สั้นมาก เช่น `Move`, `Hold Fire`, `Collect Drops`
3. กล่องดรอปแรกควรเก็บง่ายและเห็นชัด
4. จบ wave แล้วเข้าตลาดรอบแรกแบบง่าย มีของให้เลือกน้อย

สิ่งที่ผู้เล่นควรเรียนรู้โดยไม่ต้องอ่านข้อความเยอะ:

- จอยซ้ายคือเคลื่อน
- ปุ่มขวาคือยิงค้างได้
- เหรียญมีค่า
- ของดรอปต้องรีบเก็บ
- market คือช่วงอัปเกรดก่อน wave ถัดไป

## 4.3 Early Run: Waves 1-3

อารมณ์ที่ควรได้:

- "เออ เกมนี้เล่นง่าย"
- "เริ่มเห็นแล้วว่าแต่ละคนเก่งอะไร"

โครงสร้างช่วงต้น:

- Wave 1: basic enemies อย่างเดียว
- Wave 2: เริ่มมี fast enemies ให้คนต้องขยับจริง
- Wave 3: ส่ง tank ตัวแรกเพื่อสอน focus fire

นี่สอดคล้องกับ queue ปัจจุบันใน [`src/game/enemy.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/enemy.rs)

สิ่งที่ต้องเกิดในเชิงอารมณ์:

- ผู้เล่นเห็นว่าศัตรูมี role ต่างกัน
- เริ่มช่วยกัน call target
- เริ่มรู้ว่าบางคนควรเก็บ coin มากกว่า บางคนควรรีบอัป weapon

## 4.4 Mid Run: Waves 4-7

นี่คือช่วงที่เกมต้องเปลี่ยนจาก "เล่นได้" เป็น "ต้องเล่นเป็นทีม"

แกนของ journey:

- spawn หนาแน่นขึ้น
- มีศัตรูหลายชนิดพร้อมกัน
- ของดรอปเริ่มต้องแย่ง decision ว่าใครควรเก็บ
- market เริ่มเป็นเรื่อง build ไม่ใช่แค่ heal

สิ่งที่ควรเพิ่มใน phase นี้:

- elite variants หรือ mini-objectives ในบาง wave
- event banner เช่น `Meteor Field`, `Ambush`, `Salvage Rush`
- formation moments เช่น ศัตรูบีบให้ทั้งทีมย้ายฝั่งพร้อมกัน

ความรู้สึกที่ต้องเกิด:

- คนในห้องเริ่มเรียกชื่อกัน
- เริ่มมี "คนแบกดาเมจ", "คนรอดเก่ง", "คนเก็บของเก่ง"

## 4.5 Market Journey

ตอนนี้ market ในโค้ดเป็นจุดพักและซื้อของพื้นฐานได้แล้ว
แต่ใน journey ที่ดี market ต้องมีหน้าที่ 4 อย่าง:

1. ให้หายใจหลังคลื่นต่อสู้
2. ให้คุยและวางแผน
3. ให้ build diverge
4. ให้ anticipation สำหรับ wave ถัดไป

### สูตร market ที่เหมาะกับเกมนี้

ทุก market ควรตอบ 3 คำถาม:

1. ตอนนี้ทีมกำลังจะตายเพราะอะไร
2. คนไหนควรเก่งขึ้นก่อน
3. wave หน้าอยากเล่นแบบไหน

### ทางออกเชิงระบบ

item ปัจจุบัน:

- `weapon_up`
- `hp_restore`
- `max_hp_up`

สิ่งที่ควรเพิ่มต่อในอนาคต:

- `magnet` ดูดเหรียญ/ไอเท็มไกลขึ้น
- `shield_pulse` กันตาย 1 ครั้งต่อ wave
- `pierce_shot` ยิงทะลุศัตรูบางชนิด
- `coin_amp` ได้ coin เพิ่ม แต่ hp ต่ำลง
- `medic_link` เก็บ heal แล้วกระจายเล็กน้อยให้ทั้งทีม

หลักการสำคัญ:

- market ของเกมปาร์ตี้ไม่ควรมีของเยอะเกิน
- 3 ถึง 5 choices ต่อคนกำลังดี
- ทุก item ต้องอธิบายสั้นและเห็นผลใน wave ถัดไปทันที

## 4.6 Boss / Setpiece Journey

ถ้าอยากให้เกมมี journey "ครบ" จริง ต้องมีจุดพีคที่ไม่ใช่แค่ศัตรูเยอะขึ้น

แนะนำให้แบ่ง run เป็น 3 sectors:

1. Sector A: ฝึกทีม
2. Sector B: บีบทีม
3. Sector C: ทดสอบ build และ coordination

โครงสร้างตัวอย่าง:

- Wave 3: mini-boss แรก
- Wave 6: sector boss
- Wave 9: chaos wave หรือ survival wave
- Wave 10: final boss

boss ที่เหมาะกับเกมนี้ควรอ่านง่าย:

- weak point ชัด
- arena pattern ชัด
- มีหน้าต่าง damage phase
- บังคับให้ทีมกระจายและ regroup

ไม่ควรออกแบบเป็น bullet hell ซับซ้อนเกิน เพราะมือถือเป็น input device และ target audience น่าจะกว้าง

## 4.7 Failure Journey

จุดแพ้ของเกมนี้ไม่ควรให้ความรู้สึกว่า "โดนลงโทษ"
แต่ควรให้ความรู้สึกว่า "เกือบผ่านแล้ว เอาใหม่อีกรอบ"

หลังแพ้ควรมี:

- recap สั้นมาก
- ใคร damage เยอะสุด
- ใครเก็บ coin เยอะสุด
- มาถึง sector/wave ไหน
- ปุ่ม `Play Again`

สิ่งสำคัญคือ turnaround ต้องไว

ถ้าหลัง Game Over ต้องอธิบายเยอะหรือ reload ช้า เกมปาร์ตี้จะหลุด momentum ทันที

## 4.8 Post-Match Retention

ถ้าอยากให้เกมนี้ไม่จบแค่ "เล่นครั้งเดียว"
ต้องมีเหตุผลให้กลับมาอีกใน session ถัดไป

แนะนำ retention 2 ชั้น:

### ชั้นที่ 1: ต่อรอบในวงเดิม

- high score
- best wave
- MVP cards แบบขำๆ
- unlock ship colors / badges / call signs

### ชั้นที่ 2: กลับมาอีกวัน

- meta unlock แบบบาง
- enemy codex
- unlock mutators
- weekly challenge seed

ข้อควรระวัง:

- อย่าให้ meta grind กลบความสนุกของรอบพื้นฐาน
- รอบแรกต้องสนุกแม้ยังไม่ได้ unlock อะไร

## 5. Recommended Run Structure

สำหรับเวอร์ชันแรกที่ดี ควรเล็ง session ต่อรอบประมาณ 18-25 นาที

### โครงสร้างเวลา

1. Lobby + join: 1-3 นาที
2. Early game: 4-5 นาที
3. Mid game: 6-8 นาที
4. Late game + bosses: 6-8 นาที
5. Results + restart: 1 นาที

ข้อดี:

- สั้นพอสำหรับเล่นหลายรอบ
- ยาวพอให้ build มีความหมาย
- เหมาะกับกลุ่มเพื่อนและงานสังสรรค์

## 6. What This Project Already Has

ฐานที่ดีอยู่แล้ว:

- phase loop ชัดใน [`src/game/mod.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/mod.rs)
- player state รองรับ hp, max hp, coins, weapon level ใน [`src/game/state.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/state.rs)
- market economy พื้นฐานมีแล้วใน [`src/game/market.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/market.rs)
- mobile controller base พร้อมใน [`static/index.html`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/static/index.html)
- enemy escalation พื้นฐานมีแล้วใน [`src/game/enemy.rs`](/Users/kanokpichasonsmacbookair/Documents/GitHub/Games/Game001/src/game/enemy.rs)

สรุปคือโปรเจคนี้ไม่ได้เริ่มจากศูนย์แล้ว
สิ่งที่ขาดไม่ใช่ core loop แต่คือ "journey polish" และ "session identity"

## 7. Recommended Next Design Steps

ลำดับที่ควรทำต่อ:

1. ทำ lobby ให้มีชีวิต
   - join แล้วเห็นชื่อ, สี, ready state, input test ทันที

2. ทำ first-time onboarding ให้เกิดใน Wave 1-2
   - ใช้ event text สั้นๆ แทน tutorial แยก

3. ยกระดับ market ให้เป็น build moment
   - เพิ่ม item role-based อีก 3-5 ชิ้น

4. เพิ่ม sector structure
   - mini-boss, boss, special event wave

5. ทำ game-over recap
   - ให้แพ้แล้วอยากเริ่มใหม่ทันที

6. ค่อยเติม meta progression บางๆ
   - badges, cosmetics, challenge modes

## 8. Final Recommendation

ถ้าต้องนิยามเกมนี้ในประโยคเดียว:

**"เกมยิงยาน co-op บนจอเดียว ที่เข้าเล่นง่ายแบบ party game แต่มี progression และ teamwork พอให้เล่นวนหลายรอบ"**

ดังนั้น journey ที่เหมาะที่สุดไม่ใช่:

- shmup ฮาร์ดคอร์จัด
- roguelike ซับซ้อนเกินจำเป็น
- management game แบบแบ่ง station เต็มรูปแบบ

แต่ควรเป็น:

- เข้าง่าย
- อ่านง่าย
- คุยกันเยอะ
- build ชัด
- แพ้แล้วกดเล่นใหม่ไว

## Sources

- Asteroid Base, `Lovers in a Dangerous Spacetime`: https://www.asteroidbase.com/dangerous-spacetime/
- Asteroid Base, `Lovers is now on Amazon Luna!`: https://www.asteroidbase.com/news/lovers-is-now-on-amazon-luna/
- Steam, `Lovers in a Dangerous Spacetime`: https://store.steampowered.com/app/252110/Lovers_in_a_Dangerous_Spacetime/
- Steam, `Sky Force Reloaded`: https://store.steampowered.com/app/667600/Sky_Force_Reloaded/
- Spaceteam official site: https://spaceteam.ca/
- App Store, `Spaceteam`: https://apps.apple.com/uz/app/spaceteam/id570510529
- Steam, `Ship of Fools`: https://store.steampowered.com/app/1286580/Ship_of_Fools/
