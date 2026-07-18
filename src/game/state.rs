use qrcode::{Color as QrColor, QrCode};

use super::bullet::Bullet;
use super::enemy::{build_wave_queue, Enemy, Pending, SpawnKind};
use super::item::Item;

// ── Player ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    None,
    Vanguard,
    Guardian,
    Salvager,
}

impl Role {
    pub fn label(&self) -> &'static str {
        match self {
            Role::None => "None",
            Role::Vanguard => "Vanguard",
            Role::Guardian => "Guardian",
            Role::Salvager => "Salvager",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            Role::None => "NONE",
            Role::Vanguard => "VGD",
            Role::Guardian => "GRD",
            Role::Salvager => "SLV",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Role::None => Role::Vanguard,
            Role::Vanguard => Role::Guardian,
            Role::Guardian => Role::Salvager,
            Role::Salvager => Role::None,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Role::None => Role::Salvager,
            Role::Vanguard => Role::None,
            Role::Guardian => Role::Vanguard,
            Role::Salvager => Role::Guardian,
        }
    }

    pub fn base_max_hp(&self) -> u8 {
        match self {
            Role::Vanguard => 2,
            Role::Guardian => 4,
            Role::Salvager | Role::None => 3,
        }
    }

    pub fn fire_cooldown_multiplier(&self) -> f32 {
        match self {
            Role::Vanguard => 0.82,
            Role::Guardian | Role::Salvager | Role::None => 1.0,
        }
    }

    pub fn invincibility_seconds(&self) -> f32 {
        match self {
            Role::Guardian => 2.8,
            Role::Vanguard | Role::Salvager | Role::None => 2.0,
        }
    }

    pub fn pickup_radius(&self) -> f32 {
        match self {
            Role::Salvager => 24.0,
            Role::Vanguard | Role::Guardian | Role::None => 15.0,
        }
    }

    pub fn kill_coin_bonus(&self) -> u32 {
        match self {
            Role::Salvager => 2,
            Role::Vanguard | Role::Guardian | Role::None => 0,
        }
    }

    pub fn drop_coin_bonus(&self) -> u32 {
        match self {
            Role::Salvager => 8,
            Role::Vanguard | Role::Guardian | Role::None => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u8,
    pub name: String,
    pub connected: bool,
    pub ready: bool,
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub hp: u8,
    pub max_hp: u8,
    pub score: u32,
    pub coins: u32,
    pub weapon_level: u8,
    pub role: Role,
    pub fire_cooldown: f32,
    pub invincible: f32, // วินาทีที่เหลือสำหรับ invincibility หลังโดนตี
    pub firing: bool,    // fire intent จาก phone input
    pub alive: bool,
}

impl Player {
    pub fn new(id: u8, x: f32, y: f32) -> Self {
        Self {
            id, x, y,
            name: format!("P{}", id + 1),
            connected: true,
            ready: false,
            vel_x: 0.0, vel_y: 0.0,
            hp: 3, max_hp: 3,
            score: 0, coins: 0,
            weapon_level: 1,
            role: Role::None,
            fire_cooldown: 0.0,
            invincible: 0.0,
            firing: false,
            alive: true,
        }
    }
}

// ── Particle ─────────────────────────────────────────────────────────────────

pub struct Particle {
    pub x: f32, pub y: f32,
    pub vx: f32, pub vy: f32,
    pub life: f32,      // เหลือกี่วินาที
    pub max_life: f32,
    pub r: u8, pub g: u8, pub b: u8,
    pub size: f32,
}

impl Particle {
    pub fn update(&mut self, dt: f32) { self.x += self.vx * dt; self.y += self.vy * dt; self.life -= dt; }
    pub fn alive(&self) -> bool { self.life > 0.0 }
    pub fn alpha(&self) -> u8 { ((self.life / self.max_life) * 255.0) as u8 }
}

// ── Phase / Wave ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Lobby,
    Playing,
    Market,
    GameOver,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    Classic,
    Convoy,
}

#[derive(Debug, Clone)]
pub struct ConvoyCore {
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub radius: f32,
    pub alive: bool,
}

pub struct WaveState {
    pub wave: u32,
    /// enemies ที่รอ spawn
    pub pending: Vec<Pending>,
    /// countdown ก่อน spawn ตัวถัดไป
    pub spawn_timer: f32,
    /// ช่วงเวลาระหว่าง spawn (วินาที)
    pub spawn_interval: f32,
    /// countdown หลัง wave clear ก่อนขึ้น Market
    pub clear_timer: f32,
    /// true = กำลัง count down clear_timer
    pub clearing: bool,
}

impl WaveState {
    pub fn new() -> Self {
        Self {
            wave: 0,
            pending: Vec::new(),
            spawn_timer: 0.0,
            spawn_interval: 0.8,
            clear_timer: 0.0,
            clearing: false,
        }
    }

    /// เริ่ม wave ใหม่
    pub fn start(&mut self, wave: u32, screen_w: f32) {
        self.wave = wave;
        self.pending = build_wave_queue(wave, screen_w);
        self.spawn_timer = 0.5; // หน่วงเล็กน้อยก่อน spawn แรก
        self.spawn_interval = (0.9 - wave as f32 * 0.05).max(0.3);
        self.clearing = false;
        self.clear_timer = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub final_wave: u32,
    pub top_score_player: Option<u8>,
    pub top_score: u32,
    pub top_coins_player: Option<u8>,
    pub top_coins: u32,
    pub surviving_players: u8,
    pub player_count: u8,
    pub mode: GameMode,
    pub convoy_core_remaining_hp: Option<i32>,
}

// ── GameState ────────────────────────────────────────────────────────────────

pub struct GameState {
    pub players: Vec<Player>,
    pub bullets: Vec<Bullet>,
    pub enemies: Vec<Enemy>,
    pub particles: Vec<Particle>,
    pub items: Vec<Item>,
    pub phase: Phase,
    pub wave: WaveState,
    pub screen_w: f32,
    pub screen_h: f32,
    pub stars: Vec<(f32, f32)>, // background star positions
    pub market_timer: f32,       // วินาทีที่เหลือใน Market phase
    pub market_sent: bool,       // ส่ง offers ไป phone แล้วหรือยัง
    pub host_url: String,        // URL ที่ player ต้องเปิด
    pub qr_grid: Vec<Vec<bool>>, // QR code pixels (true = dark module)
    pub last_summary: Option<RunSummary>,
    pub mode: GameMode,          // classic หรือ convoy
    pub convoy_core: Option<ConvoyCore>,
    pub role_draft_cursor: usize,
}

impl GameState {
    pub fn new(screen_w: f32, screen_h: f32, host_url: &str) -> Self {
        // สร้าง star field แบบ static
        let stars = (0..200)
            .map(|i| {
                let x = (i as f32 * 137.5) % screen_w;
                let y = (i as f32 * 97.3)  % screen_h;
                (x, y)
            })
            .collect();

        // Generate QR code grid (true = dark module)
        let qr_grid = QrCode::new(host_url.as_bytes())
            .ok()
            .map(|code| {
                let w = code.width();
                (0..w)
                    .map(|row| (0..w).map(|col| code[(col, row)] == QrColor::Dark).collect())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            players: Vec::new(),
            bullets: Vec::new(),
            enemies: Vec::new(),
            particles: Vec::new(),
            items: Vec::new(),
            phase: Phase::Lobby,
            wave: WaveState::new(),
            screen_w,
            screen_h,
            stars,
            market_timer: 0.0,
            market_sent: false,
            host_url: host_url.to_string(),
            qr_grid,
            last_summary: None,
            mode: GameMode::Classic,
            convoy_core: None,
            role_draft_cursor: 0,
        }
    }

    pub fn is_convoy_mode(&self) -> bool {
        self.mode == GameMode::Convoy
    }

    pub fn connected_count(&self) -> usize {
        self.players.iter().filter(|p| p.connected).count()
    }

    pub fn ready_count(&self) -> usize {
        self.players.iter().filter(|p| p.connected && p.ready).count()
    }

    pub fn all_connected_players_ready(&self) -> bool {
        let connected = self.connected_count();
        connected > 0 && self.ready_count() == connected
    }

    /// Spawn player ใหม่ แบ่งตำแหน่งล่างจอเท่าๆ กัน
    pub fn add_player(&mut self, id: u8) {
        if self.players.iter().any(|p| p.id == id) { return; }
        let slot  = self.players.len() as f32;
        let x = self.screen_w / 9.0 * (slot + 1.0);
        let y = self.screen_h - 80.0;
        self.players.push(Player::new(id, x, y));
    }

    pub fn build_run_summary(&self) -> RunSummary {
        let top_score_player = self.players.iter().max_by_key(|p| p.score).map(|p| p.id);
        let top_score = self.players.iter().map(|p| p.score).max().unwrap_or(0);

        let top_coins_player = self.players.iter().max_by_key(|p| p.coins).map(|p| p.id);
        let top_coins = self.players.iter().map(|p| p.coins).max().unwrap_or(0);

        RunSummary {
            final_wave: self.wave.wave,
            top_score_player,
            top_score,
            top_coins_player,
            top_coins,
            surviving_players: self.players.iter().filter(|p| p.alive).count() as u8,
            player_count: self.players.len() as u8,
            mode: self.mode.clone(),
            convoy_core_remaining_hp: self.convoy_core.as_ref().map(|core| core.hp.max(0)),
        }
    }

    /// Spawn particle explosion ที่ตำแหน่งที่กำหนด
    pub fn explode(&mut self, cx: f32, cy: f32, r: u8, g: u8, b: u8, count: usize) {
        use std::f32::consts::TAU;
        for i in 0..count {
            let angle = (i as f32 / count as f32) * TAU + (cy * 0.1) % TAU;
            let speed = 60.0 + (cx * 0.3 + i as f32 * 17.0) % 120.0;
            let life  = 0.4 + (i as f32 * 0.03) % 0.4;
            self.particles.push(Particle {
                x: cx, y: cy,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life, max_life: life, r, g, b,
                size: 2.0 + (i % 3) as f32,
            });
        }
    }

    /// Reset ว่าค่าเริ่มต้น (restart game)
    pub fn reset(&mut self) {
        self.players.clear();
        self.bullets.clear();
        self.enemies.clear();
        self.particles.clear();
        self.items.clear();
        self.phase = Phase::Lobby;
        self.wave = WaveState::new();
        self.market_timer = 0.0;
        self.market_sent = false;
        self.last_summary = None;
        self.role_draft_cursor = 0;
        if self.mode == GameMode::Convoy {
            self.convoy_core = None;
        }
    }

    pub fn restart_to_lobby(&mut self) {
        self.reset_run_preserve_players();
        self.phase = Phase::Lobby;
    }

    pub fn quick_replay(&mut self) {
        self.reset_run_preserve_players();
        self.wave.start(1, self.screen_w);
        self.phase = Phase::Playing;
    }

    fn reset_run_preserve_players(&mut self) {
        self.bullets.clear();
        self.enemies.clear();
        self.particles.clear();
        self.items.clear();
        self.wave = WaveState::new();
        self.market_timer = 0.0;
        self.market_sent = false;
        self.last_summary = None;
        if self.mode == GameMode::Convoy {
            self.convoy_core = None;
        }

        for (slot, p) in self.players.iter_mut().enumerate() {
            let x = self.screen_w / 9.0 * (slot as f32 + 1.0);
            let y = self.screen_h - 80.0;
            p.x = x;
            p.y = y;
            p.vel_x = 0.0;
            p.vel_y = 0.0;
            p.max_hp = p.role.base_max_hp();
            p.hp = p.max_hp;
            p.score = 0;
            p.coins = 0;
            p.weapon_level = 1;
            p.fire_cooldown = 0.0;
            p.invincible = 0.0;
            p.firing = false;
            p.alive = true;
            p.connected = true;
        }
    }

    /// Spawn enemy จาก Pending
    pub fn spawn_from_pending(enemies: &mut Vec<Enemy>, p: Pending, wave: u32) {
        let e = match p.kind {
            SpawnKind::Basic => Enemy::basic(p.x, wave),
            SpawnKind::Fast  => Enemy::fast(p.x, wave),
            SpawnKind::Tank  => Enemy::tank(p.x, wave),
        };
        enemies.push(e);
    }
}
