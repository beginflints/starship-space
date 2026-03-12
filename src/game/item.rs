use macroquad::prelude::Color;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ItemKind {
    HpUp,      // ฟื้น HP 1 ดวง
    WeaponUp,  // weapon level +1
    CoinBoost, // เหรียญ +20
}

pub struct Item {
    pub kind: ItemKind,
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub radius: f32,
    pub alive: bool,
    pub phase: f32,    // ใช้กับ bobbing animation
    pub lifetime: f32, // วินาทีที่เหลือก่อน despawn
}

const FALL_SPEED: f32 = 28.0;
const LIFETIME:   f32 = 9.0;

impl Item {
    pub fn new(kind: ItemKind, x: f32, y: f32) -> Self {
        Self { kind, x, y, vy: FALL_SPEED, radius: 13.0, alive: true, phase: 0.0, lifetime: LIFETIME }
    }

    pub fn update(&mut self, dt: f32, screen_h: f32) {
        self.phase    += dt;
        self.lifetime -= dt;
        // bob ขึ้นลง + ตกช้าๆ
        self.y += self.vy * dt + (self.phase * 2.8).sin() * 0.4;
        if self.lifetime <= 0.0 || self.y > screen_h + 30.0 {
            self.alive = false;
        }
    }

    pub fn color(&self) -> Color {
        match self.kind {
            ItemKind::HpUp      => Color::from_rgba(255, 80,  80,  255),
            ItemKind::WeaponUp  => Color::from_rgba(80,  180, 255, 255),
            ItemKind::CoinBoost => Color::from_rgba(255, 215, 0,   255),
        }
    }

    pub fn label(&self) -> &str {
        match self.kind {
            ItemKind::HpUp      => "+HP",
            ItemKind::WeaponUp  => "WPN",
            ItemKind::CoinBoost => "$$$",
        }
    }
}

/// ตัดสินใจ drop item จาก enemy ที่ตาย (30% drop chance รวม)
pub fn roll_drop(enemy_x: f32, enemy_y: f32, wave: u32) -> Option<ItemKind> {
    // pseudo-random โดยใช้ position + wave เป็น seed (ไม่ต้องการ rng state)
    let h = (enemy_x as u32)
        .wrapping_mul(2654435761)
        .wrapping_add((enemy_y as u32).wrapping_mul(2246822519))
        .wrapping_add(wave.wrapping_mul(1664525));
    match h % 20 {
        0..=4  => Some(ItemKind::CoinBoost), // 25%
        5..=6  => Some(ItemKind::HpUp),      // 10%
        7      => Some(ItemKind::WeaponUp),   // 5%
        _      => None,
    }
}
