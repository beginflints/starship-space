/// ══════════════════════════════════════════════════════════════════════════
/// effects.rs  —  VFX parameter tables
///
/// เพิ่ม weapon level ใหม่ → เพิ่ม WeaponFx entry ใน WEAPON_FX
/// เพิ่ม damage stage ใหม่ → insert DamageStage ใน DAMAGE_STAGES (เรียง threshold ↑)
/// ══════════════════════════════════════════════════════════════════════════

// ── Weapon-level VFX ─────────────────────────────────────────────────────────

/// พารามิเตอร์ VFX ทั้งหมดของแต่ละ weapon level
pub struct WeaponFx {
    // ── muzzle flash (spawned ตอนยิง) ───────────────────────────────────────
    pub flash_count:      usize,
    pub flash_rgb:        (u8, u8, u8),
    pub flash_spread:     f32,         // half-angle spread (radians)
    pub flash_speed_base: f32,
    pub flash_speed_step: f32,
    pub flash_life_base:  f32,
    pub flash_life_step:  f32,

    // ── bullet trail (1 particle / bullet / frame) ───────────────────────────
    pub trail_rgb: (u8, u8, u8),

    // ── bullet renderer ──────────────────────────────────────────────────────
    /// extra radius ของ outer glow
    pub outer_glow_extra:  f32,
    /// fixed RGBA สำหรับ outer glow;
    /// None = ใช้ player color ที่ outer_glow_player_alpha
    pub outer_glow_rgba:   Option<(u8, u8, u8, u8)>,
    pub outer_glow_player_alpha: f32,
    /// extra radius ของ mid ring (0.0 = ไม่มี ring)
    pub ring_extra:        f32,
    pub ring_player_alpha: f32,
    /// ratio ของ bullet.radius สำหรับ core dot (0.0 = ไม่มี dot)
    pub core_ratio:        f32,
    pub core_rgba:         Option<(u8, u8, u8, u8)>,
}

/// ตาราง WeaponFx index 0 = level 1, index 1 = level 2, …
/// level ที่เกินขอบตาราง → ใช้ entry สุดท้ายเป็น fallback
#[rustfmt::skip]
pub static WEAPON_FX: &[WeaponFx] = &[
    // ── Level 1: single golden shot ─────────────────────────────────────────
    WeaponFx {
        flash_count:      4,
        flash_rgb:        (255, 200,  80),
        flash_spread:     0.45,
        flash_speed_base: 75.0,  flash_speed_step: 25.0,
        flash_life_base:  0.060, flash_life_step:  0.010,
        trail_rgb:        (200, 160,  60),
        outer_glow_extra: 2.5, outer_glow_rgba: None, outer_glow_player_alpha: 0.20,
        ring_extra: 0.0, ring_player_alpha: 0.0,
        core_ratio: 0.0, core_rgba: None,
    },
    // ── Level 2: cyan 3-way burst ─────────────────────────────────────────────
    WeaponFx {
        flash_count:      6,
        flash_rgb:        (100, 220, 255),
        flash_spread:     0.75,
        flash_speed_base: 80.0,  flash_speed_step: 22.0,
        flash_life_base:  0.070, flash_life_step:  0.010,
        trail_rgb:        ( 60, 180, 255),
        outer_glow_extra: 5.0, outer_glow_rgba: Some((80, 200, 255, 55)), outer_glow_player_alpha: 0.0,
        ring_extra: 1.5, ring_player_alpha: 0.55,
        core_ratio: 0.42, core_rgba: Some((255, 255, 255, 255)),
    },
    // ── Level 3: orange 5-way spread ─────────────────────────────────────────
    WeaponFx {
        flash_count:      9,
        flash_rgb:        (255, 140,  30),
        flash_spread:     1.20,
        flash_speed_base: 85.0,  flash_speed_step: 20.0,
        flash_life_base:  0.070, flash_life_step:  0.009,
        trail_rgb:        (255, 120,  20),
        outer_glow_extra: 7.0, outer_glow_rgba: Some((255, 120, 20, 45)), outer_glow_player_alpha: 0.0,
        ring_extra: 2.5, ring_player_alpha: 0.60,
        core_ratio: 0.50, core_rgba: Some((255, 240, 140, 255)),
    },
    // ── Level 4: purple plasma ────────────────────────────────────────────────
    WeaponFx {
        flash_count:      11,
        flash_rgb:        (200,  80, 255),
        flash_spread:     1.50,
        flash_speed_base: 90.0,  flash_speed_step: 20.0,
        flash_life_base:  0.080, flash_life_step:  0.008,
        trail_rgb:        (180,  60, 255),
        outer_glow_extra: 9.0, outer_glow_rgba: Some((180, 40, 255, 55)), outer_glow_player_alpha: 0.0,
        ring_extra: 3.5, ring_player_alpha: 0.65,
        core_ratio: 0.55, core_rgba: Some((255, 220, 255, 255)),
    },
    // ── Level 5: white overcharge (max level) ─────────────────────────────────
    WeaponFx {
        flash_count:      14,
        flash_rgb:        (255, 255, 220),
        flash_spread:     1.80,
        flash_speed_base: 100.0, flash_speed_step: 18.0,
        flash_life_base:  0.090, flash_life_step:  0.007,
        trail_rgb:        (240, 240, 200),
        outer_glow_extra: 11.0, outer_glow_rgba: Some((255, 255, 200, 50)), outer_glow_player_alpha: 0.0,
        ring_extra: 4.5, ring_player_alpha: 0.70,
        core_ratio: 0.60, core_rgba: Some((255, 255, 255, 255)),
    },
];

/// คืน WeaponFx สำหรับ weapon_level ที่กำหนด (1-indexed)
/// level ที่เกิน table → ใช้ entry สุดท้าย
pub fn weapon_fx(level: u8) -> &'static WeaponFx {
    let idx = (level as usize).saturating_sub(1);
    WEAPON_FX.get(idx).unwrap_or_else(|| WEAPON_FX.last().unwrap())
}

// ── Ship damage stages ────────────────────────────────────────────────────────

/// Visual stage ที่แสดงเมื่อ hp_ratio ≤ threshold
/// *** เรียง threshold จากน้อยไปมาก (most severe first) ***
pub struct DamageStage {
    /// stage นี้ active เมื่อ hp_ratio ≤ threshold
    pub hp_ratio_threshold: f32,

    // aura วงกลมหลัง ship
    pub aura_radius_mult: f32,
    pub aura_rgb:         (u8, u8, u8),
    pub aura_alpha_base:  u8,
    pub aura_pulse_hz:    f32,  // 0 = static
    pub aura_pulse_amp:   u8,   // + added on top of aura_alpha_base when pulsing

    // cockpit overlay
    pub cockpit_rgb:         (u8, u8, u8),
    pub cockpit_alpha_base:  u8,
    pub cockpit_pulse_hz:    f32,
    pub cockpit_pulse_amp:   u8,

    // รอยร้าว: (x1_mult, y1_mult, x2_mult, y2_mult) × size
    pub cracks: &'static [(f32, f32, f32, f32)],

    // smoke / fire particle
    pub smoke_rgb:       (u8, u8, u8),
    /// > 0 = เพิ่ม sin-flicker บน green channel (ไฟ); 0 = ควันนิ่ง
    pub smoke_flicker_amp: u8,
    /// sin gate threshold (-1..1); ยิ่งต่ำ emit บ่อยขึ้น
    pub emit_threshold:  f32,
    pub smoke_size:      f32,
}

/// ตาราง DamageStage เรียง threshold ASC — most severe อยู่บน
#[rustfmt::skip]
pub static DAMAGE_STAGES: &[DamageStage] = &[
    // ── Stage 2 — CRITICAL (≤ 34 % HP) : pulsing red + fire ─────────────────
    DamageStage {
        hp_ratio_threshold: 0.34,
        aura_radius_mult:   1.65,
        aura_rgb:           (255,  40,  40),
        aura_alpha_base:    50,  aura_pulse_hz:  6.0, aura_pulse_amp:  70,
        cockpit_rgb:        (255,  50,  50),
        cockpit_alpha_base: 170, cockpit_pulse_hz: 9.0, cockpit_pulse_amp: 65,
        cracks: &[
            (-0.15, -0.08,  0.20,  0.22),
            ( 0.06, -0.26, -0.18,  0.06),
        ],
        smoke_rgb:          (255,  70,  10),
        smoke_flicker_amp:  40,
        emit_threshold:     0.30,
        smoke_size:         4.0,
    },
    // ── Stage 1 — DAMAGED (34–99 % HP) : static orange + gray smoke ──────────
    DamageStage {
        hp_ratio_threshold: 0.99,
        aura_radius_mult:   1.45,
        aura_rgb:           (255, 140,  40),
        aura_alpha_base:    38,  aura_pulse_hz:  0.0, aura_pulse_amp:   0,
        cockpit_rgb:        (255, 160,  50),
        cockpit_alpha_base: 190, cockpit_pulse_hz: 0.0, cockpit_pulse_amp: 0,
        cracks: &[(-0.12, -0.05, 0.18, 0.20)],
        smoke_rgb:          (120, 115, 115),
        smoke_flicker_amp:  0,
        emit_threshold:     0.55,
        smoke_size:         3.0,
    },
];

/// คืน DamageStage ที่รุนแรงที่สุดสำหรับ hp_ratio ที่กำหนด
/// คืน None เมื่อ HP เต็ม (ไม่มี stage ที่ตรงเงื่อนไข)
pub fn damage_stage(hp_ratio: f32) -> Option<&'static DamageStage> {
    // DAMAGE_STAGES เรียง threshold ASC → entry แรกที่ hp_ratio ≤ threshold คือ most severe
    DAMAGE_STAGES.iter().find(|s| hp_ratio <= s.hp_ratio_threshold)
}
