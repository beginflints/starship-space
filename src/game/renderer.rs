use macroquad::prelude::*;

use super::enemy::EnemyKind;
use super::state::{GameMode, GameState, Phase};

const SHIP_R: f32 = 16.0;

const PLAYER_COLORS: [Color; 8] = [
    BLUE, GREEN, RED, YELLOW, PURPLE, ORANGE, PINK, SKYBLUE,
];

pub fn draw(state: &GameState) {
    clear_background(Color::from_rgba(5, 5, 18, 255));

    draw_stars(state);

    match state.phase {
        Phase::Lobby    => draw_lobby(state),
        Phase::Playing  => draw_playing(state),
        Phase::Market   => draw_market(state),
        Phase::GameOver => draw_game_over(state),
    }
}

// ── Stars ────────────────────────────────────────────────────────────────────

fn draw_stars(state: &GameState) {
    for (i, &(x, y)) in state.stars.iter().enumerate() {
        let bright = if i % 5 == 0 { 200u8 } else { 100 };
        let size   = if i % 7 == 0 { 2.0 } else { 1.0 };
        draw_circle(x, y, size, Color::from_rgba(bright, bright, bright, 255));
    }
}

// ── Lobby ────────────────────────────────────────────────────────────────────

fn draw_lobby(state: &GameState) {
    let sw = state.screen_w;
    let sh = state.screen_h;
    let connected = state.connected_count();
    let ready = state.ready_count();

    let left_cx  = sw * 0.37; // center x สำหรับ text ทางซ้าย
    let right_cx = sw * 0.76; // center x สำหรับ QR ทางขวา

    // ── Title (full width) ───────────────────────────────────────────────────
    draw_centered("STARSHIP SPACE",    sw / 2.0, sh * 0.12, 72.0, WHITE);
    draw_centered("MULTIPLAYER ARCADE", sw / 2.0, sh * 0.20, 30.0, GRAY);

    // ── Left: connection info ────────────────────────────────────────────────
    draw_centered("Open your phone browser:", left_cx, sh * 0.36, 22.0, LIGHTGRAY);
    draw_centered(&state.host_url,            left_cx, sh * 0.44, 28.0, YELLOW);

    let blink = Color::from_rgba(255, 255, 255, ((get_time() * 2.0).sin() * 100.0 + 155.0) as u8);
    if state.players.is_empty() {
        draw_centered("Waiting for players...", left_cx, sh * 0.60, 26.0, blink);
    } else if state.all_connected_players_ready() {
        draw_centered("Press [SPACE] to start", left_cx, sh * 0.60, 28.0, blink);
    } else {
        draw_centered("Waiting for ready pilots...", left_cx, sh * 0.60, 26.0, blink);
    }

    draw_centered(
        &format!("Players connected: {}/8", connected),
        left_cx, sh * 0.70, 26.0, GREEN,
    );
    draw_centered(
        &format!("Ready to launch: {}/{}", ready, connected.max(1)),
        left_cx, sh * 0.75, 20.0, if ready == connected && connected > 0 { SKYBLUE } else { ORANGE },
    );
    
    // Mode indicator
    if state.is_convoy_mode() {
        draw_centered("Mode: CONVOY", left_cx, sh * 0.81, 24.0, MAGENTA);
    } else {
        draw_centered("Mode: CLASSIC", left_cx, sh * 0.81, 24.0, SKYBLUE);
    }
    draw_centered("Press [M] to change mode", left_cx, sh * 0.86, 18.0, GRAY);

    // ── Right: QR code ───────────────────────────────────────────────────────
    if !state.qr_grid.is_empty() {
        let n      = state.qr_grid.len() as f32;
        let cell   = (sh * 0.50 / n).floor().max(3.0); // cell size in pixels
        let total  = cell * n;
        let margin = 8.0;
        let qx     = right_cx - total / 2.0;
        let qy     = sh * 0.26;

        // white border background
        draw_rectangle(qx - margin, qy - margin, total + margin * 2.0, total + margin * 2.0, WHITE);

        for (row, cols) in state.qr_grid.iter().enumerate() {
            for (col, &dark) in cols.iter().enumerate() {
                if dark {
                    draw_rectangle(
                        qx + col as f32 * cell,
                        qy + row as f32 * cell,
                        cell, cell,
                        BLACK,
                    );
                }
            }
        }

        draw_centered(
            "Scan to play!",
            right_cx, qy + total + margin * 2.0 + 18.0, 22.0, LIGHTGRAY,
        );
    }

    // ── Lobby roster (bottom) ────────────────────────────────────────────────
    let panel_x = sw * 0.12;
    let panel_y = sh * 0.87;
    let panel_w = sw * 0.76;
    let panel_h = sh * 0.11;
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::from_rgba(8, 12, 24, 210));
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, Color::from_rgba(255, 255, 255, 32));
    draw_text("Pilot Roster", panel_x + 16.0, panel_y + 24.0, 22.0, LIGHTGRAY);

    if state.players.is_empty() {
        draw_text("No connected pilots yet", panel_x + 16.0, panel_y + 52.0, 18.0, GRAY);
    } else {
        let columns = if state.players.len() > 4 { 2 } else { 1 };
        let rows_per_column = ((state.players.len() + columns - 1) / columns).max(1);
        let gap = 16.0;
        let inner_x = panel_x + 16.0;
        let inner_y = panel_y + 34.0;
        let inner_w = panel_w - 32.0;
        let card_w = if columns == 1 {
            inner_w
        } else {
            (inner_w - gap) / 2.0
        };
        let row_h = 26.0;

        for (idx, player) in state.players.iter().enumerate() {
            let column = idx / rows_per_column;
            let row = idx % rows_per_column;
            let x = inner_x + column as f32 * (card_w + gap);
            let y = inner_y + row as f32 * row_h;
            let color = PLAYER_COLORS[player.id as usize % PLAYER_COLORS.len()];
            draw_lobby_player_row(x, y, card_w, color, player);
        }
    }
}

// ── Playing ──────────────────────────────────────────────────────────────────

fn draw_playing(state: &GameState) {
    if state.is_convoy_mode() {
        draw_convoy_core(state);
    }

    draw_enemies(state);
    draw_items(state);
    draw_bullets(state);
    draw_players(state);
    draw_particles(state);
    draw_hud_playing(state);
}

fn draw_convoy_core(state: &GameState) {
    if let Some(core) = &state.convoy_core {
        if !core.alive { return; }

        let pulse = ((get_time() * 3.0).sin() * 0.5 + 0.5) as f32;
        let c = Color::from_rgba(100, 200, 255, 255);
        let glow = Color::from_rgba(100, 200, 255, (100.0 + 50.0 * pulse) as u8);
        
        // Draw core
        draw_circle(core.x, core.y, core.radius + 8.0, glow);
        draw_poly(core.x, core.y, 6, core.radius, get_time() as f32 * 20.0, c);
        draw_poly(core.x, core.y, 6, core.radius * 0.7, -get_time() as f32 * 15.0, WHITE);
        
        // HP Bar
        let bar_w = core.radius * 2.5;
        let bar_h = 8.0;
        let bx = core.x - bar_w / 2.0;
        let by = core.y + core.radius + 15.0;
        draw_rectangle(bx, by, bar_w, bar_h, Color::from_rgba(80, 0, 0, 200));
        let hp_frac = core.hp as f32 / core.max_hp as f32;
        draw_rectangle(bx, by, bar_w * hp_frac, bar_h, if hp_frac > 0.3 { GREEN } else { RED });
        
        // Label
        draw_centered("DEFEND THE CORE", core.x, core.y - core.radius - 20.0, 22.0, YELLOW);
    }
}

fn draw_enemies(state: &GameState) {
    for e in &state.enemies {
        let c = e.color();
        match e.kind {
            EnemyKind::Basic => {
                // Diamond shape
                let r = e.radius;
                draw_triangle(
                    Vec2::new(e.x, e.y - r),
                    Vec2::new(e.x + r * 0.7, e.y),
                    Vec2::new(e.x, e.y + r),
                    c,
                );
                draw_triangle(
                    Vec2::new(e.x, e.y - r),
                    Vec2::new(e.x - r * 0.7, e.y),
                    Vec2::new(e.x, e.y + r),
                    c,
                );
            }
            EnemyKind::Fast => {
                // Thin forward triangle
                let r = e.radius;
                draw_triangle(
                    Vec2::new(e.x, e.y + r),
                    Vec2::new(e.x - r * 0.5, e.y - r * 0.7),
                    Vec2::new(e.x + r * 0.5, e.y - r * 0.7),
                    c,
                );
            }
            EnemyKind::Tank => {
                // Large circle with inner ring
                draw_circle(e.x, e.y, e.radius, c);
                draw_circle(e.x, e.y, e.radius * 0.55,
                    Color::new(c.r * 0.5, c.g * 0.5, c.b * 0.5, 1.0));
                // HP bar above
                let bar_w = e.radius * 2.2;
                let bar_h = 5.0;
                let bx = e.x - bar_w / 2.0;
                let by = e.y - e.radius - 10.0;
                draw_rectangle(bx, by, bar_w, bar_h, Color::from_rgba(80, 0, 0, 200));
                let hp_frac = e.hp as f32 / e.max_hp as f32;
                draw_rectangle(bx, by, bar_w * hp_frac, bar_h, GREEN);
            }
        }
    }
}

fn draw_items(state: &GameState) {
    for item in &state.items {
        let c = item.color();

        // กระพริบเมื่อ lifetime เหลือน้อย
        let alpha = if item.lifetime < 3.0 {
            (item.phase * 6.0).sin() * 0.5 + 0.5
        } else {
            1.0_f32
        };

        let glow = Color::new(c.r, c.g, c.b, alpha * 0.25);
        let fill = Color::new(c.r, c.g, c.b, alpha);
        let label_c = Color::new(1.0, 1.0, 1.0, alpha);

        draw_circle(item.x, item.y, item.radius + 4.0, glow);
        draw_circle(item.x, item.y, item.radius, fill);
        draw_centered(item.label(), item.x, item.y + 4.0, 13.0, label_c);
    }
}

fn draw_bullets(state: &GameState) {
    for b in &state.bullets {
        let pc = PLAYER_COLORS[b.owner_id as usize % 8];
        let fx = super::effects::weapon_fx(b.weapon_level);

        // outer glow
        let glow_c = match fx.outer_glow_rgba {
            Some((r, g, bl, a)) => Color::from_rgba(r, g, bl, a),
            None => Color::new(pc.r, pc.g, pc.b, fx.outer_glow_player_alpha),
        };
        draw_circle(b.x, b.y, b.radius + fx.outer_glow_extra, glow_c);

        // mid ring (player colour, optional)
        if fx.ring_extra > 0.0 {
            draw_circle(b.x, b.y, b.radius + fx.ring_extra,
                Color::new(pc.r, pc.g, pc.b, fx.ring_player_alpha));
        }

        // bullet body
        draw_circle(b.x, b.y, b.radius, pc);

        // core dot (optional)
        if fx.core_ratio > 0.0 {
            if let Some((r, g, bl, a)) = fx.core_rgba {
                draw_circle(b.x, b.y, b.radius * fx.core_ratio, Color::from_rgba(r, g, bl, a));
            }
        }
    }
}

fn draw_players(state: &GameState) {
    for (i, p) in state.players.iter().enumerate() {
        if !p.alive { continue; }

        // ถ้า invincible ให้กระพริบ
        if p.invincible > 0.0 {
            let blink = (p.invincible * 8.0) as u32 % 2 == 0;
            if blink { continue; }
        }

        let c = PLAYER_COLORS[i % 8];
        let hp_ratio = p.hp as f32 / p.max_hp.max(1) as f32;

        if p.hp < p.max_hp {
            draw_ship_damaged(p.x, p.y, SHIP_R, c, hp_ratio);
        } else {
            draw_ship(p.x, p.y, SHIP_R, c);
        }

        // name tag
        draw_centered_at(&p.name, p.x, p.y - SHIP_R - 8.0, 15.0, c);
    }
}

fn draw_particles(state: &GameState) {
    for p in &state.particles {
        let a = p.alpha();
        let c = Color::from_rgba(p.r, p.g, p.b, a);
        draw_circle(p.x, p.y, p.size, c);
    }
}

fn draw_hud_playing(state: &GameState) {
    let sw = state.screen_w;
    let sh = state.screen_h;

    // Wave indicator
    draw_text(
        &format!("WAVE {}", state.wave.wave),
        20.0, 35.0, 30.0, WHITE,
    );

    // Enemy count
    let alive_e = state.enemies.len() + state.wave.pending.len();
    draw_text(
        &format!("Enemies: {}", alive_e),
        20.0, 65.0, 20.0, GRAY,
    );

    // Bottom score bar
    let bar_h = 44.0;
    let y = sh - bar_h;
    draw_rectangle(0.0, y, sw, bar_h, Color::from_rgba(0, 0, 0, 200));

    for (i, p) in state.players.iter().enumerate() {
        let col_w = sw / 8.0;
        let x = i as f32 * col_w + 8.0;
        let c = if p.alive { PLAYER_COLORS[i % 8] } else { GRAY };

        draw_ship_icon(x + 10.0, y + 14.0, 10.0, c);

        let hp_str: String = "♥".repeat(p.hp as usize);
        draw_text(&hp_str, x + 26.0, y + 18.0, 14.0, RED);
        draw_text(&format!("{}", p.score), x + 8.0, y + 38.0, 18.0, c);
    }
}

// ── Market ───────────────────────────────────────────────────────────────────

fn draw_market(state: &GameState) {
    let sw = state.screen_w;
    let sh = state.screen_h;

    // dim overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 160));

    draw_centered("MARKET", sw / 2.0, sh * 0.13, 64.0, YELLOW);
    draw_centered("Use your phone to upgrade!", sw / 2.0, sh * 0.22, 26.0, LIGHTGRAY);

    // player stats
    for (i, p) in state.players.iter().enumerate() {
        let col = sw / 8.0;
        let x = (i as f32 + 0.5) * col;
        let c = PLAYER_COLORS[i % 8];

        let status = if p.alive { "" } else { " (dead)" };
        draw_centered(&format!("{}{}", p.name, status), x, sh * 0.36, 20.0, c);
        draw_centered(&format!("Coins: {}", p.coins),      x, sh * 0.43, 18.0, YELLOW);
        draw_centered(&format!("HP: {}/{}", p.hp, p.max_hp), x, sh * 0.50, 16.0, RED);
        draw_centered(&format!("Wpn Lv{}", p.weapon_level), x, sh * 0.56, 16.0, SKYBLUE);
    }

    // items on screen (dropped but not picked up)
    draw_items(state);

    // countdown
    if state.market_timer > 0.0 {
        draw_centered(
            &format!("Next wave in {:.0}s", state.market_timer.ceil()),
            sw / 2.0, sh * 0.88, 28.0, WHITE,
        );
        draw_centered("[ENTER] to skip", sw / 2.0, sh * 0.94, 20.0, GRAY);
    }
}

// ── Game Over ────────────────────────────────────────────────────────────────

fn draw_game_over(state: &GameState) {
    let sw = state.screen_w;
    let sh = state.screen_h;

    draw_rectangle(0.0, 0.0, sw, sh, Color::from_rgba(0, 0, 0, 160));
    draw_centered("GAME OVER", sw / 2.0, sh * 0.16, 76.0, RED);

    let panel_w = sw * 0.72;
    let panel_h = sh * 0.48;
    let panel_x = (sw - panel_w) * 0.5;
    let panel_y = sh * 0.25;
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::from_rgba(10, 14, 30, 220));
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 3.0, Color::from_rgba(255, 255, 255, 60));

    if let Some(summary) = &state.last_summary {
        draw_centered(
            &format!("Reached Wave {}", summary.final_wave),
            sw / 2.0,
            panel_y + 46.0,
            34.0,
            WHITE,
        );
        let mode_label = match summary.mode {
            GameMode::Classic => "Classic Run",
            GameMode::Convoy => "Convoy Run",
        };
        draw_centered(mode_label, sw / 2.0, panel_y + 76.0, 20.0, LIGHTGRAY);

        if let Some(core_hp) = summary.convoy_core_remaining_hp {
            let core_text = if core_hp > 0 {
                format!("Core survived with {} HP", core_hp)
            } else {
                "Core destroyed".to_string()
            };
            draw_centered(&core_text, sw / 2.0, panel_y + 100.0, 20.0, ORANGE);
        }

        let card_w = panel_w * 0.26;
        let card_h = 118.0;
        let gap = panel_w * 0.055;
        let cards_y = panel_y + 124.0;
        let cards_x = [
            panel_x + gap,
            panel_x + gap * 2.0 + card_w,
            panel_x + gap * 3.0 + card_w * 2.0,
        ];

        let score_name = summary
            .top_score_player
            .and_then(|pid| state.players.iter().find(|p| p.id == pid))
            .map(|p| p.name.as_str())
            .unwrap_or("None");
        draw_summary_card(
            cards_x[0],
            cards_y,
            card_w,
            card_h,
            "Top Score",
            &format!("{}", summary.top_score),
            score_name,
            YELLOW,
        );

        let coins_name = summary
            .top_coins_player
            .and_then(|pid| state.players.iter().find(|p| p.id == pid))
            .map(|p| p.name.as_str())
            .unwrap_or("None");
        draw_summary_card(
            cards_x[1],
            cards_y,
            card_w,
            card_h,
            "Coin Leader",
            &format!("{}", summary.top_coins),
            coins_name,
            GOLD,
        );

        draw_summary_card(
            cards_x[2],
            cards_y,
            card_w,
            card_h,
            "Crew Status",
            &format!("{}/{}", summary.surviving_players, summary.player_count),
            "survived",
            SKYBLUE,
        );

        let list_y = cards_y + card_h + 24.0;
        draw_centered("Pilot Recap", sw / 2.0, list_y, 24.0, LIGHTGRAY);

        let row_h = 24.0;
        let mut players: Vec<_> = state.players.iter().collect();
        players.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| b.coins.cmp(&a.coins))
                .then_with(|| a.id.cmp(&b.id))
        });

        let column_count = if players.len() > 4 { 2 } else { 1 };
        let rows_per_column = ((players.len() + column_count - 1) / column_count).max(1);
        let column_gap = 24.0;
        let total_w = panel_w - 88.0;
        let column_w = if column_count == 1 {
            total_w
        } else {
            (total_w - column_gap) / 2.0
        };
        let list_x = panel_x + 44.0;

        for column in 0..column_count {
            let x = list_x + column as f32 * (column_w + column_gap);
            draw_player_recap_header(x, list_y + 24.0, column_w);
        }

        for (idx, player) in players.iter().enumerate() {
            let column = idx / rows_per_column;
            let row = idx % rows_per_column;
            let x = list_x + column as f32 * (column_w + column_gap);
            let y = list_y + 50.0 + row as f32 * row_h;
            draw_player_recap_row(x, y, column_w, idx + 1, player);
        }
        
    } else {
        draw_centered("Run summary unavailable", sw / 2.0, panel_y + panel_h * 0.5, 28.0, GRAY);
    }

    draw_centered("Press [SPACE] for quick replay", sw / 2.0, sh * 0.80, 28.0, WHITE);

    let blink = Color::from_rgba(255, 255, 255, ((get_time() * 2.0).sin() * 100.0 + 155.0) as u8);
    draw_centered("Press [R] to return to lobby", sw / 2.0, sh * 0.86, 24.0, blink);
    draw_centered("Press [M] to change mode", sw / 2.0, sh * 0.91, 20.0, GRAY);

    draw_particles(state);
}

// ── Ship helpers ─────────────────────────────────────────────────────────────

fn draw_ship(x: f32, y: f32, size: f32, color: Color) {
    let tip   = Vec2::new(x, y - size);
    let left  = Vec2::new(x - size * 0.7, y + size * 0.6);
    let right = Vec2::new(x + size * 0.7, y + size * 0.6);
    let mid   = Vec2::new(x, y + size * 0.15);

    draw_triangle(tip, left, right, color);
    // engine glow
    draw_triangle(left, right, mid,
        Color::new(color.r, color.g, color.b, 0.27));
    // cockpit dot
    draw_circle(x, y - size * 0.3, size * 0.18, WHITE);
}

/// วาด ship พร้อม damage overlay ตาม hp_ratio (table-driven via effects::DAMAGE_STAGES)
fn draw_ship_damaged(x: f32, y: f32, size: f32, color: Color, hp_ratio: f32) {
    let Some(stage) = super::effects::damage_stage(hp_ratio) else { return; };

    // ── aura ──────────────────────────────────────────────────────────────────
    let aura_a = if stage.aura_pulse_hz > 0.0 {
        let pulse = ((get_time() * stage.aura_pulse_hz as f64).sin() * 0.5 + 0.5) as f32;
        (stage.aura_alpha_base as f32 + stage.aura_pulse_amp as f32 * pulse) as u8
    } else {
        stage.aura_alpha_base
    };
    let (ar, ag, ab) = stage.aura_rgb;
    draw_circle(x, y, size * stage.aura_radius_mult, Color::from_rgba(ar, ag, ab, aura_a));

    // ── base ship ─────────────────────────────────────────────────────────────
    draw_ship(x, y, size, color);

    // ── cockpit overlay ───────────────────────────────────────────────────────
    let cockpit_a = if stage.cockpit_pulse_hz > 0.0 {
        let pulse = ((get_time() * stage.cockpit_pulse_hz as f64).sin() * 0.5 + 0.5) as f32;
        (stage.cockpit_alpha_base as f32 + stage.cockpit_pulse_amp as f32 * pulse) as u8
    } else {
        stage.cockpit_alpha_base
    };
    let (cr, cg, cb) = stage.cockpit_rgb;
    draw_circle(x, y - size * 0.3, size * 0.19, Color::from_rgba(cr, cg, cb, cockpit_a));

    // ── crack lines ───────────────────────────────────────────────────────────
    for &(x1m, y1m, x2m, y2m) in stage.cracks {
        draw_line(
            x + size * x1m, y + size * y1m,
            x + size * x2m, y + size * y2m,
            1.5, Color::from_rgba(255, 210, 80, 158),
        );
    }
}

fn draw_ship_icon(x: f32, y: f32, size: f32, color: Color) {
    draw_ship(x, y, size, color);
}

fn draw_lobby_player_row(
    x: f32,
    y: f32,
    w: f32,
    color: Color,
    player: &super::state::Player,
) {
    draw_rectangle(x, y - 16.0, w, 22.0, Color::from_rgba(255, 255, 255, 10));
    draw_ship_icon(x + 12.0, y - 4.0, 10.0, color);
    draw_text(&player.name, x + 28.0, y, 18.0, color);

    let connected_label = if player.connected { "CONNECTED" } else { "OFFLINE" };
    let connected_color = if player.connected { GREEN } else { RED };
    let ready_label = if player.ready { "READY" } else { "JOINING" };
    let ready_color = if player.ready { SKYBLUE } else { ORANGE };

    draw_text(connected_label, x + w * 0.52, y, 16.0, connected_color);
    draw_text(ready_label, x + w * 0.78, y, 16.0, ready_color);
}

fn draw_summary_card(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    title: &str,
    value: &str,
    subtitle: &str,
    accent: Color,
) {
    draw_rectangle(x, y, w, h, Color::from_rgba(22, 28, 48, 230));
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(accent.r, accent.g, accent.b, 0.55));
    draw_centered(title, x + w * 0.5, y + 24.0, 22.0, LIGHTGRAY);
    draw_centered(value, x + w * 0.5, y + 64.0, 34.0, accent);
    draw_centered(subtitle, x + w * 0.5, y + 95.0, 20.0, WHITE);
}

fn draw_player_recap_header(x: f32, y: f32, w: f32) {
    draw_text("Pilot", x + 52.0, y, 16.0, GRAY);
    draw_text("Status", x + w * 0.46, y, 16.0, GRAY);
    draw_text("Score", x + w * 0.62, y, 16.0, GRAY);
    draw_text("Coins", x + w * 0.80, y, 16.0, GRAY);
}

fn draw_player_recap_row(x: f32, y: f32, w: f32, rank: usize, player: &super::state::Player) {
    let color = PLAYER_COLORS[player.id as usize % PLAYER_COLORS.len()];
    let row_bg = if rank % 2 == 0 {
        Color::from_rgba(255, 255, 255, 10)
    } else {
        Color::from_rgba(255, 255, 255, 18)
    };

    draw_rectangle(x, y - 18.0, w, 24.0, row_bg);
    draw_text(&format!("#{}", rank), x + 10.0, y, 18.0, LIGHTGRAY);
    draw_text(&player.name, x + 52.0, y, 18.0, color);

    let status = if player.alive { "Survived" } else { "Down" };
    draw_text(status, x + w * 0.46, y, 18.0, if player.alive { GREEN } else { RED });
    draw_text(&format!("Score {}", player.score), x + w * 0.62, y, 18.0, YELLOW);
    draw_text(&format!("Coins {}", player.coins), x + w * 0.80, y, 18.0, GOLD);
}

// ── Text helpers ─────────────────────────────────────────────────────────────

fn draw_centered(text: &str, cx: f32, cy: f32, font_size: f32, color: Color) {
    let d = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, cx - d.width / 2.0, cy + d.height / 2.0, font_size, color);
}

fn draw_centered_at(text: &str, cx: f32, y: f32, font_size: f32, color: Color) {
    let d = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, cx - d.width / 2.0, y, font_size, color);
}
