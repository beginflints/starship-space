/// กระสุนหนึ่งลูก
pub struct Bullet {
    pub owner_id: u8,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub damage: u8,
    pub alive: bool,
    pub weapon_level: u8, // เก็บไว้เพื่อ renderer ใช้แสดงผล
}

impl Bullet {
    /// Spawn กระสุนตาม weapon level — คืน Vec เพราะอาจยิงหลายลูกพร้อมกัน
    pub fn spawn(owner_id: u8, x: f32, y: f32, weapon_level: u8) -> Vec<Self> {
        let speed = 520.0_f32;
        match weapon_level {
            1 => vec![Self::mk(owner_id, x, y, 0.0, -speed, 4.0, 1, weapon_level)],
            2 => {
                let a = 15_f32.to_radians();
                vec![
                    Self::mk(owner_id, x, y, -a.sin() * speed, -a.cos() * speed, 4.0, 1, weapon_level),
                    Self::mk(owner_id, x, y,  0.0,              -speed,           4.0, 1, weapon_level),
                    Self::mk(owner_id, x, y,  a.sin() * speed, -a.cos() * speed, 4.0, 1, weapon_level),
                ]
            }
            _ => {
                // level 3+: 5-way spread
                let angles = [-30_f32, -15.0, 0.0, 15.0, 30.0];
                angles
                    .iter()
                    .map(|deg| {
                        let a = deg.to_radians();
                        Self::mk(owner_id, x, y, a.sin() * speed, -a.cos() * speed, 4.0, 2, weapon_level)
                    })
                    .collect()
            }
        }
    }

    fn mk(owner_id: u8, x: f32, y: f32, vx: f32, vy: f32, radius: f32, damage: u8, weapon_level: u8) -> Self {
        Self { owner_id, x, y, vx, vy, radius, damage, alive: true, weapon_level }
    }

    pub fn update(&mut self, dt: f32, screen_w: f32, screen_h: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        if self.y < -20.0 || self.y > screen_h + 20.0
            || self.x < -20.0 || self.x > screen_w + 20.0
        {
            self.alive = false;
        }
    }
}
