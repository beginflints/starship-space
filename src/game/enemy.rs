use macroquad::prelude::Color;

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyKind {
    Basic,  // ลงตรงๆ ช้า
    Fast,   // ลงตรงๆ เร็ว เลือดน้อย
    Tank,   // zigzag เลือดเยอะ
}

pub struct Enemy {
    pub kind: EnemyKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub radius: f32,
    pub score_value: u32,
    pub coin_value: u32,
    pub alive: bool,
    pub phase: f32, // ใช้กับ zigzag
}

impl Enemy {
    pub fn basic(x: f32, wave: u32) -> Self {
        let spd = 55.0 + wave as f32 * 8.0;
        Self::new(EnemyKind::Basic, x, -30.0, 0.0, spd, 2, 14.0, 100, 5)
    }

    pub fn fast(x: f32, wave: u32) -> Self {
        let spd = 120.0 + wave as f32 * 12.0;
        Self::new(EnemyKind::Fast, x, -20.0, 0.0, spd, 1, 9.0, 150, 8)
    }

    pub fn tank(x: f32, wave: u32) -> Self {
        let spd = 35.0 + wave as f32 * 5.0;
        Self::new(EnemyKind::Tank, x, -50.0, 0.0, spd, 8 + wave as i32 * 2, 22.0, 400, 25)
    }

    fn new(
        kind: EnemyKind, x: f32, y: f32, vx: f32, vy: f32,
        hp: i32, radius: f32, score_value: u32, coin_value: u32,
    ) -> Self {
        Self {
            kind, x, y, vx, vy, hp, max_hp: hp,
            radius, score_value, coin_value, alive: true, phase: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, screen_w: f32, screen_h: f32) {
        self.phase += dt;

        // zigzag สำหรับ Tank
        if self.kind == EnemyKind::Tank {
            self.vx = self.phase.sin() * 90.0;
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // bounce ขอบซ้าย-ขวา
        if self.x < self.radius        { self.x = self.radius; self.vx = self.vx.abs(); }
        if self.x > screen_w - self.radius { self.x = screen_w - self.radius; self.vx = -self.vx.abs(); }

        if self.y > screen_h + 60.0 {
            self.alive = false;
        }
    }

    /// สีของ enemy แต่ละ type
    pub fn color(&self) -> Color {
        match self.kind {
            EnemyKind::Basic => Color::from_rgba(220, 80,  80,  255),
            EnemyKind::Fast  => Color::from_rgba(255, 200, 50,  255),
            EnemyKind::Tank  => Color::from_rgba(160, 60,  220, 255),
        }
    }
}

// ── Wave definition ────────────────────────────────────────────────────────

/// ประเภท enemy ที่จะ spawn
#[derive(Clone, Copy)]
pub enum SpawnKind { Basic, Fast, Tank }

/// Enemy ที่รอ spawn (อยู่ใน queue)
#[derive(Clone, Copy)]
pub struct Pending { pub kind: SpawnKind, pub x: f32 }

/// สร้าง spawn queue สำหรับ wave ที่กำหนด
pub fn build_wave_queue(wave: u32, screen_w: f32) -> Vec<Pending> {
    let mut q = Vec::new();
    let col = |n: usize| -> f32 { screen_w / (n as f32 + 1.0) };

    match wave {
        1 => {
            for i in 1..=6 { q.push(Pending { kind: SpawnKind::Basic, x: col(6) * i as f32 }); }
        }
        2 => {
            for i in 1..=4 { q.push(Pending { kind: SpawnKind::Basic, x: col(4) * i as f32 }); }
            for i in 1..=3 { q.push(Pending { kind: SpawnKind::Fast,  x: col(3) * i as f32 }); }
        }
        3 => {
            for i in 1..=4 { q.push(Pending { kind: SpawnKind::Fast, x: col(4) * i as f32 }); }
            q.push(Pending { kind: SpawnKind::Tank, x: screen_w * 0.25 });
            q.push(Pending { kind: SpawnKind::Tank, x: screen_w * 0.75 });
        }
        n => {
            // wave 4+ escalate
            let basics = 3 + n;
            let fasts  = 2 + n / 2;
            let tanks  = n / 3;
            for i in 1..=basics { q.push(Pending { kind: SpawnKind::Basic, x: col(basics as usize) * i as f32 }); }
            for i in 1..=fasts  { q.push(Pending { kind: SpawnKind::Fast,  x: col(fasts  as usize) * i as f32 }); }
            for i in 1..=tanks  { q.push(Pending { kind: SpawnKind::Tank,  x: col(tanks.max(1) as usize) * i as f32 }); }
        }
    }
    q
}
