mod audio;
mod bullet;
mod effects;
mod enemy;
mod item;
mod market;
mod renderer;
mod state;

use macroquad::prelude::*;
use tokio::sync::{broadcast, mpsc};

use audio::GameAudio;
use crate::server::{GameEvent, PlayerInput};
use bullet::Bullet;
use item::{roll_drop, Item, ItemKind};
use state::{GameState, Particle, Phase, GameMode, ConvoyCore};

const SHIP_SPEED: f32 = 280.0;
const FIRE_COOLDOWN: f32 = 0.22; // วินาที
const MARKET_DURATION: f32 = 12.0; // วินาทีที่ market เปิด
const WAVE_CLEAR_PAUSE: f32 = 2.5;
const RESPAWN_DURATION: f32 = 3.0;

pub async fn run(
    mut input_rx: mpsc::Receiver<PlayerInput>,
    event_tx: broadcast::Sender<GameEvent>,
    host_url: String,
) {
    let mut gs = GameState::new(screen_width(), screen_height(), &host_url);

    // Wait one frame so the macOS audio system is fully initialised
    // before we call load_sound_from_bytes / play_sound.
    next_frame().await;

    let mut audio = GameAudio::load().await;

    loop {
        let dt = get_frame_time().min(0.05); // cap ที่ 50ms กัน spiral of death

        // ── 1. drain phone inputs ────────────────────────────────────────────
        drain_inputs(&mut input_rx, &mut gs, &event_tx);

        // ── 2. update per phase ──────────────────────────────────────────────
        match gs.phase {
            Phase::Lobby => update_lobby(&mut gs, &event_tx, &audio),

            Phase::Playing => {
                update_players(&mut gs, dt, &audio);
                update_wave_spawner(&mut gs, dt, &event_tx);
                update_bullets(&mut gs, dt);
                update_enemies(&mut gs, dt);
                resolve_collisions(&mut gs, &event_tx, &audio);
                update_items(&mut gs, dt, &event_tx, &audio);
                update_respawns(&mut gs, dt, &event_tx);
                update_particles(&mut gs, dt);
                check_wave_complete(&mut gs, dt, &event_tx, &audio);
                check_game_over(&mut gs, &event_tx, &audio);
                broadcast_player_states(&gs, &event_tx);
            }

            Phase::Market => {
                gs.market_timer -= dt;
                update_particles(&mut gs, dt);
                update_items(&mut gs, dt, &event_tx, &audio);
                broadcast_player_states(&gs, &event_tx);

                // ส่ง market offers ให้ phone ครั้งแรกเมื่อเข้า Market phase
                if !gs.market_sent {
                    gs.market_sent = true;
                    let offers_list: Vec<_> = gs.players.iter()
                        .filter(|p| p.alive)
                        .map(|p| (p.id, market::get_offers(p)))
                        .collect();
                    for (pid, items) in offers_list {
                        let _ = event_tx.send(GameEvent::SendMarket { player_id: pid, items });
                    }
                }

                if gs.market_timer <= 0.0 || is_key_pressed(KeyCode::Enter) {
                    start_next_wave(&mut gs, &event_tx, &audio);
                }
            }

            Phase::GameOver => {
                update_particles(&mut gs, dt);
                if is_key_pressed(KeyCode::Space) && !gs.players.is_empty() {
                    gs.quick_replay();
                    audio.play_wave_start();
                    let _ = event_tx.send(GameEvent::Broadcast("Quick replay - Wave 1!".into()));
                } else if is_key_pressed(KeyCode::M) && !gs.players.is_empty() {
                    gs.mode = match gs.mode {
                        GameMode::Classic => GameMode::Convoy,
                        GameMode::Convoy => GameMode::Classic,
                    };
                    gs.restart_to_lobby();
                    let _ = event_tx.send(GameEvent::Broadcast("Mode changed - back to lobby.".into()));
                } else if is_key_pressed(KeyCode::R) && !gs.players.is_empty() {
                    gs.restart_to_lobby();
                    let _ = event_tx.send(GameEvent::Broadcast("Back to lobby.".into()));
                }
            }
        }

        // ── 3. draw ──────────────────────────────────────────────────────────
        renderer::draw(&gs);

        // ── 4. BGM — switch track when phase changes ─────────────────────────
        audio.update_bgm(&gs.phase);

        // ── 5. debug shortcuts ───────────────────────────────────────────────
        #[cfg(debug_assertions)]
        {
            if is_key_pressed(KeyCode::D) {
                let id = gs.players.len() as u8;
                gs.add_player(id);
            }
            if is_key_pressed(KeyCode::K) {
                // kill all enemies (debug skip wave)
                for e in gs.enemies.iter_mut() { e.alive = false; }
            }
        }

        next_frame().await;
    }
}

// ── Input ────────────────────────────────────────────────────────────────────

fn drain_inputs(
    rx: &mut mpsc::Receiver<PlayerInput>,
    gs: &mut GameState,
    event_tx: &broadcast::Sender<GameEvent>,
) {
    while let Ok(input) = rx.try_recv() {
        // ── Disconnect ────────────────────────────────────────────────────────
        if input.disconnect {
            if let Some(idx) = gs.players.iter().position(|p| p.id == input.player_id) {
                let name = gs.players[idx].name.clone();
                gs.players.remove(idx);
                println!("[game] player {} ('{}') left", input.player_id, name);
                let _ = event_tx.send(GameEvent::Broadcast(format!("{} left the game", name)));
            }
            // ถ้า Playing/Market และไม่มี player เหลือ → กลับ Lobby
            if gs.players.is_empty()
                && (gs.phase == Phase::Playing || gs.phase == Phase::Market)
            {
                gs.reset();
                println!("[game] all players left — returning to Lobby");
            }
            continue;
        }

        // ── เพิ่ม player ใหม่ถ้ายังไม่มีใน game ─────────────────────────────
        if !gs.players.iter().any(|p| p.id == input.player_id) {
            gs.add_player(input.player_id);
            println!("[game] player {} joined", input.player_id);
        }

        // ── อัพเดตชื่อจาก Join message ───────────────────────────────────────
        if let Some(ref name) = input.name {
            if let Some(p) = gs.players.iter_mut().find(|p| p.id == input.player_id) {
                let display = if name.is_empty() {
                    format!("P{}", input.player_id + 1)
                } else {
                    name.clone()
                };
                p.name = display.clone();
                p.connected = true;
                p.ready = true;
                println!("[game] player {} name set to '{}'", input.player_id, display);
                let _ = event_tx.send(GameEvent::Broadcast(format!("{} joined!", display)));
            }
            continue; // Join message ไม่มี joy/fire ที่ต้องใช้
        }

        // ── บันทึก ship design จาก phone designer ────────────────────────────
        if let Some(design) = input.ship_design {
            if let Some(p) = gs.players.iter_mut().find(|p| p.id == input.player_id) {
                println!("[game] player {} ship design saved ({} cells)", input.player_id, design.cells.len());
                p.ship_design = Some(design);
            }
            continue;
        }

        // ── อัพเดต movement + fire intent ────────────────────────────────────
        if let Some(p) = gs.players.iter_mut().find(|p| p.id == input.player_id) {
            p.connected = true;
            p.vel_x  = input.joy[0] * SHIP_SPEED;
            p.vel_y  = input.joy[1] * SHIP_SPEED;
            p.firing = input.fire;
        }

        // ── จัดการ market purchase ────────────────────────────────────────────
        if let Some(item_id) = input.buy_item {
            if gs.phase == Phase::Market {
                if let Some(idx) = gs.players.iter().position(|p| p.id == input.player_id) {
                    if market::apply_purchase(&mut gs.players[idx], &item_id) {
                        let pid = gs.players[idx].id;
                        let pname = gs.players[idx].name.clone();
                        let _ = event_tx.send(GameEvent::Broadcast(format!(
                            "{} upgraded: {}!", pname, item_id.replace('_', " ")
                        )));
                        
                        // ส่ง PlayerState เพื่อซิงค์เหรียญและ HP/Weapon ล่าสุดกลับไปที่มือถือ
                        let _ = event_tx.send(GameEvent::PlayerState {
                            player_id: pid,
                            hp: gs.players[idx].hp,
                            max_hp: gs.players[idx].max_hp,
                            score: gs.players[idx].score,
                            coins: gs.players[idx].coins,
                            weapon_level: gs.players[idx].weapon_level,
                            respawning: gs.players[idx].is_respawning,
                            respawn_seconds: gs.players[idx].respawn_timer.max(0.0),
                        });

                        // ส่ง market offers ที่อัพเดตแล้วกลับไป phone
                        let new_offers = market::get_offers(&gs.players[idx]);
                        let _ = event_tx.send(GameEvent::SendMarket { player_id: pid, items: new_offers });
                    }
                }
            }
        }
    }
}

fn update_respawns(
    gs: &mut GameState,
    dt: f32,
    event_tx: &broadcast::Sender<GameEvent>,
) {
    let mut respawned: Vec<usize> = Vec::new();

    for (idx, player) in gs.players.iter_mut().enumerate() {
        if !player.is_respawning { continue; }

        player.respawn_timer = (player.respawn_timer - dt).max(0.0);
        if player.respawn_timer <= 0.0 {
            respawned.push(idx);
        }
    }

    for idx in respawned {
        let (x, y) = gs.player_spawn_point(idx);
        let player = &mut gs.players[idx];
        player.x = x;
        player.y = y;
        player.vel_x = 0.0;
        player.vel_y = 0.0;
        player.hp = player.max_hp;
        player.alive = true;
        player.invincible = player.role.invincibility_seconds();
        player.firing = false;
        player.fire_cooldown = 0.0;
        player.is_respawning = false;
        player.respawn_timer = 0.0;

        let _ = event_tx.send(GameEvent::Broadcast(format!("{} re-entered the fight!", player.name)));
    }
}

// ── Lobby ────────────────────────────────────────────────────────────────────

fn update_lobby(gs: &mut GameState, event_tx: &broadcast::Sender<GameEvent>, audio: &GameAudio) {
    // ── Role Draft: host เลือกผู้เล่นและสลับ role จาก keyboard ─────────────────
    if !gs.players.is_empty() {
        gs.role_draft_cursor = gs.role_draft_cursor.min(gs.players.len() - 1);

        if is_key_pressed(KeyCode::Left) {
            gs.role_draft_cursor = if gs.role_draft_cursor == 0 {
                gs.players.len() - 1
            } else {
                gs.role_draft_cursor - 1
            };
        } else if is_key_pressed(KeyCode::Right) {
            gs.role_draft_cursor = (gs.role_draft_cursor + 1) % gs.players.len();
        }

        if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::E) {
            let player = &mut gs.players[gs.role_draft_cursor];
            player.role = if is_key_pressed(KeyCode::Q) {
                player.role.prev()
            } else {
                player.role.next()
            };
            player.max_hp = player.role.base_max_hp();
            player.hp = player.max_hp;
            player.invincible = 0.0;
            let _ = event_tx.send(GameEvent::Broadcast(format!(
                "{} role set to {}",
                player.name,
                player.role.label()
            )));
        }
    }

    if is_key_pressed(KeyCode::M) {
        gs.mode = match gs.mode {
            GameMode::Classic => GameMode::Convoy,
            GameMode::Convoy => GameMode::Classic,
        };
        // Option to broadcast mode change here if desired
    }
    
    if is_key_pressed(KeyCode::Space) && gs.all_connected_players_ready() {
        if gs.is_convoy_mode() {
            gs.convoy_core = Some(ConvoyCore {
                x: gs.screen_w / 2.0,
                y: gs.screen_h - 100.0, // placed lower on screen, above bottom HUD
                hp: 10,
                max_hp: 10,
                radius: 40.0,
                alive: true,
            });
        }
        gs.wave.start(1, gs.screen_w);
        gs.phase = Phase::Playing;
        let _ = event_tx.send(GameEvent::Broadcast("Wave 1 — GO!".into()));
        audio.play_wave_start();
    }
}

// ── Players ──────────────────────────────────────────────────────────────────

fn update_players(gs: &mut GameState, dt: f32, audio: &GameAudio) {
    let sw = gs.screen_w;
    let sh = gs.screen_h;
    let mut new_bullets: Vec<Bullet> = Vec::new();
    let mut new_particles: Vec<Particle> = Vec::new();

    for p in gs.players.iter_mut() {
        if !p.alive { continue; }

        // move
        p.x = (p.x + p.vel_x * dt).clamp(15.0, sw - 15.0);
        p.y = (p.y + p.vel_y * dt).clamp(20.0, sh - 50.0);

        // invincibility countdown
        if p.invincible > 0.0 { p.invincible -= dt; }

        // fire cooldown
        if p.fire_cooldown > 0.0 { p.fire_cooldown -= dt; }

        // spawn bullets เมื่อ firing และ cooldown หมด
        if p.firing && p.fire_cooldown <= 0.0 {
            p.fire_cooldown = FIRE_COOLDOWN * p.role.fire_cooldown_multiplier();
            let mut bs = Bullet::spawn(p.id, p.x, p.y - 20.0, p.weapon_level);
            if !bs.is_empty() {
                audio.play_shoot();

                // ── muzzle flash (table-driven) ───────────────────────────────
                let fx = effects::weapon_fx(p.weapon_level);
                let half = fx.flash_count as f32 / 2.0;
                let (fr, fg, fb) = fx.flash_rgb;
                for i in 0..fx.flash_count {
                    let angle = (i as f32 - half) / half.max(0.5) * fx.flash_spread * 0.5;
                    let speed = fx.flash_speed_base + i as f32 * fx.flash_speed_step;
                    let life  = fx.flash_life_base  + i as f32 * fx.flash_life_step;
                    new_particles.push(Particle {
                        x: p.x, y: p.y - 20.0,
                        vx:  angle.sin() * speed,
                        vy: -(angle.cos() * speed + 15.0),
                        life, max_life: life,
                        r: fr, g: fg, b: fb,
                        size: 1.5 + (i % 3) as f32 * 0.7,
                    });
                }
            }
            new_bullets.append(&mut bs);
        }

        // ── damage smoke / fire (table-driven) ───────────────────────────────
        let hp_ratio = p.hp as f32 / p.max_hp.max(1) as f32;
        if let Some(stage) = effects::damage_stage(hp_ratio) {
            let t     = get_time() as f32;
            let pid_f = p.id as f32;
            if (t * 9.0 + pid_f * 2.3).sin() > stage.emit_threshold {
                let (sr, mut sg, sb) = stage.smoke_rgb;
                if stage.smoke_flicker_amp > 0 {
                    let flicker = ((t * 14.0 + pid_f).sin() * stage.smoke_flicker_amp as f32) as u8;
                    sg = sg.saturating_add(flicker);
                }
                new_particles.push(Particle {
                    x:  p.x + (t * 4.5 + pid_f).sin() * 4.0,
                    y:  p.y + 10.0,
                    vx: (t * 3.0 + pid_f).cos() * 18.0,
                    vy: 25.0 + (t * 6.0 + pid_f).sin() * 12.0,
                    life: 0.45, max_life: 0.45,
                    r: sr, g: sg, b: sb,
                    size: stage.smoke_size,
                });
            }
        }
    }

    gs.bullets.append(&mut new_bullets);
    gs.particles.append(&mut new_particles);
}

// ── Wave spawner ─────────────────────────────────────────────────────────────

fn update_wave_spawner(
    gs: &mut GameState,
    dt: f32,
    event_tx: &broadcast::Sender<GameEvent>,
) {
    if gs.wave.clearing || gs.wave.pending.is_empty() { return; }

    gs.wave.spawn_timer -= dt;
    if gs.wave.spawn_timer <= 0.0 {
        if let Some(pending) = gs.wave.pending.pop() {
            let wave_num = gs.wave.wave;
            GameState::spawn_from_pending(&mut gs.enemies, pending, wave_num);
            gs.wave.spawn_timer = gs.wave.spawn_interval;
        }
    }

    let _ = event_tx; // silence unused warning
}

// ── Bullets ──────────────────────────────────────────────────────────────────

fn update_bullets(gs: &mut GameState, dt: f32) {
    let sw = gs.screen_w;
    let sh = gs.screen_h;
    let mut new_particles: Vec<Particle> = Vec::new();

    for bullet in gs.bullets.iter_mut() {
        if !bullet.alive { continue; }

        // ── trail particle (table-driven) ────────────────────────────────────
        let (tr, tg, tb) = effects::weapon_fx(bullet.weapon_level).trail_rgb;
        new_particles.push(Particle {
            x: bullet.x,
            y: bullet.y + bullet.radius, // slightly behind bullet tip
            vx: bullet.vx * 0.04,
            vy: bullet.vy * 0.04 + 18.0, // mostly stationary, drifts opposite to travel
            life: 0.07, max_life: 0.07,
            r: tr, g: tg, b: tb,
            size: bullet.radius * 0.65,
        });

        bullet.update(dt, sw, sh);
    }

    gs.bullets.retain(|b| b.alive);
    gs.particles.append(&mut new_particles);
}

// ── Enemies ──────────────────────────────────────────────────────────────────

fn update_enemies(gs: &mut GameState, dt: f32) {
    let sw = gs.screen_w;
    let sh = gs.screen_h;
    for e in gs.enemies.iter_mut() {
        e.update(dt, sw, sh);
    }
    gs.enemies.retain(|e| e.alive);
}

// ── Items ────────────────────────────────────────────────────────────────────

fn update_items(gs: &mut GameState, dt: f32, event_tx: &broadcast::Sender<GameEvent>, audio: &GameAudio) {
    let sh = gs.screen_h;
    for item in gs.items.iter_mut() {
        item.update(dt, sh);
    }

    // item × player collision (collect first to avoid borrow conflict)
    let mut pickups: Vec<(usize, usize)> = Vec::new(); // (item_idx, player_idx)
    for (ii, item) in gs.items.iter().enumerate() {
        if !item.alive { continue; }
        for (pi, p) in gs.players.iter().enumerate() {
            if !p.alive { continue; }
            if circles_overlap(item.x, item.y, item.radius, p.x, p.y, p.role.pickup_radius()) {
                pickups.push((ii, pi));
                break;
            }
        }
    }

    for (ii, pi) in pickups {
        let kind = gs.items[ii].kind; // ItemKind: Copy
        gs.items[ii].alive = false;

        let p = &mut gs.players[pi];
        let pickup_label = match kind {
            ItemKind::HpUp => {
                if p.hp < p.max_hp { p.hp += 1; }
                "+HP".to_string()
            }
            ItemKind::WeaponUp => {
                if p.weapon_level < 5 { p.weapon_level += 1; }
                "weapon up".to_string()
            }
            ItemKind::CoinBoost => {
                let coins = 20 + p.role.drop_coin_bonus();
                p.coins += coins;
                format!("+{} coins", coins)
            }
        };

        let pid = p.id;
        let msg = format!("P{} {}!", pid + 1, pickup_label);
        audio.play_pickup();
        let _ = event_tx.send(GameEvent::Broadcast(msg));
    }

    gs.items.retain(|i| i.alive);
}

// ── Collisions ───────────────────────────────────────────────────────────────

struct Hit {
    bullet_idx: usize,
    enemy_idx: usize,
    owner_id: u8,
    damage: u8,
}

fn resolve_collisions(gs: &mut GameState, event_tx: &broadcast::Sender<GameEvent>, audio: &GameAudio) {
    // ── Bullet × Enemy ──────────────────────────────────────────────────────
    let mut hits: Vec<Hit> = Vec::new();

    for (bi, b) in gs.bullets.iter().enumerate() {
        if !b.alive { continue; }
        for (ei, e) in gs.enemies.iter().enumerate() {
            if !e.alive { continue; }
            if circles_overlap(b.x, b.y, b.radius, e.x, e.y, e.radius) {
                hits.push(Hit { bullet_idx: bi, enemy_idx: ei, owner_id: b.owner_id, damage: b.damage });
                break; // bullet ตายที่ enemy แรกที่ชน
            }
        }
    }

    for h in &hits {
        gs.bullets[h.bullet_idx].alive = false;

        // Extract all values from the enemy in one scope so the borrow ends
        // before we call gs.explode (which needs &mut gs)
        let (ex, ey, er, eg, eb, sv, cv, dead) = {
            let e = &mut gs.enemies[h.enemy_idx];
            e.hp -= h.damage as i32;
            let dead = e.hp <= 0;
            if dead { e.alive = false; }
            let c = e.color();
            (e.x, e.y,
             (c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8,
             e.score_value, e.coin_value, dead)
        };

        let wave_num = gs.wave.wave;

        if dead {
            audio.play_explode();
            gs.explode(ex, ey, er, eg, eb, 18);

            // ให้ score + coins กับเจ้าของกระสุน
            if let Some(p) = gs.players.iter_mut().find(|p| p.id == h.owner_id) {
                p.score += sv;
                p.coins += cv + p.role.kill_coin_bonus();
            }

            // drop item
            if let Some(kind) = roll_drop(ex, ey, wave_num) {
                gs.items.push(Item::new(kind, ex, ey));
            }
        } else {
            // hit flash particle
            gs.explode(ex, ey, 255, 255, 255, 4);
        }
    }

    gs.bullets.retain(|b| b.alive);
    gs.enemies.retain(|e| e.alive);

    // ── Enemy × Player ──────────────────────────────────────────────────────
    let mut dmg_list: Vec<usize> = Vec::new(); // player index ที่โดนตี

    for e in gs.enemies.iter() {
        for (pi, p) in gs.players.iter().enumerate() {
            if !p.alive || p.is_respawning || p.invincible > 0.0 { continue; }
            if circles_overlap(e.x, e.y, e.radius, p.x, p.y, 15.0) {
                dmg_list.push(pi);
            }
        }
    }

    dmg_list.sort_unstable();
    dmg_list.dedup();

    for pi in dmg_list {
        // copy ค่าที่ต้องใช้ออกก่อน เพื่อหลีกเลี่ยง double-borrow
        let (px, py) = {
            let p = &gs.players[pi];
            (p.x, p.y)
        };

        let mut death_broadcast: Option<String> = None;
        {
            let p = &mut gs.players[pi];
            p.hp = p.hp.saturating_sub(1);
            p.invincible = p.role.invincibility_seconds();
            if p.hp == 0 {
                p.alive = false;
                p.vel_x = 0.0;
                p.vel_y = 0.0;
                p.firing = false;
                p.fire_cooldown = 0.0;
                p.invincible = 0.0;

                if gs.reinforcements > 0 {
                    gs.reinforcements -= 1;
                    p.is_respawning = true;
                    p.respawn_timer = RESPAWN_DURATION;
                    death_broadcast = Some(format!(
                        "{} down! Reinforcements {}/{}",
                        p.name, gs.reinforcements, gs.max_reinforcements
                    ));
                } else {
                    p.is_respawning = false;
                    p.respawn_timer = 0.0;
                    death_broadcast = Some(format!("{} was destroyed!", p.name));
                }
            }
        }

        if let Some(text) = death_broadcast {
            let _ = event_tx.send(GameEvent::Broadcast(text));
        }

        audio.play_player_hit();
        gs.explode(px, py, 255, 100, 100, 10);
    }

    // ── Enemy × ConvoyCore ───────────────────────────────────────────────────
    if gs.is_convoy_mode() {
        // Collect damage and hits first
        let mut core_dmg = 0;
        let mut hit_enemy_idx = Vec::new();
        let mut core_x = 0.0;
        let mut core_y = 0.0;
        let mut core_hp = 0;
        
        if let Some(core) = &gs.convoy_core {
            if core.alive {
                core_x = core.x;
                core_y = core.y;
                for (ei, e) in gs.enemies.iter().enumerate() {
                    if !e.alive { continue; }
                    if circles_overlap(e.x, e.y, e.radius, core.x, core.y, core.radius) {
                        core_dmg += 1;
                        hit_enemy_idx.push(ei);
                    }
                }
            }
        }

        // Apply damage and explode
        if core_dmg > 0 {
            if let Some(core) = gs.convoy_core.as_mut() {
                core.hp = core.hp.saturating_sub(core_dmg);
                core_hp = core.hp;
                if core.hp == 0 {
                    core.alive = false;
                }
            }
            
            if core_hp == 0 {
                let _ = event_tx.send(GameEvent::Broadcast("CONVOY CORE DESTROYED!".into()));
            } else {
                let _ = event_tx.send(GameEvent::Broadcast(format!("Core under attack! HP: {}", core_hp)));
            }
            audio.play_player_hit();
            gs.explode(core_x, core_y, 255, 150, 50, core_dmg as usize * 10);
        }

        // Destroy enemies that hit the core
        for ei in hit_enemy_idx {
            gs.enemies[ei].alive = false;
            let e = &gs.enemies[ei];
            gs.explode(e.x, e.y, 255, 100, 100, 15);
        }
    }
}

// ── Particles ────────────────────────────────────────────────────────────────

fn update_particles(gs: &mut GameState, dt: f32) {
    for p in gs.particles.iter_mut() { p.update(dt); }
    gs.particles.retain(|p| p.alive());
}

// ── Wave complete ────────────────────────────────────────────────────────────

fn check_wave_complete(
    gs: &mut GameState,
    dt: f32,
    event_tx: &broadcast::Sender<GameEvent>,
    audio: &GameAudio,
) {
    // ยังมี enemy หรือรอ spawn อยู่ → ไม่จบ
    if !gs.enemies.is_empty() || !gs.wave.pending.is_empty() { return; }

    if !gs.wave.clearing {
        gs.wave.clearing = true;
        gs.wave.clear_timer = WAVE_CLEAR_PAUSE;
        let _ = event_tx.send(GameEvent::Broadcast(format!(
            "Wave {} cleared! Get ready...", gs.wave.wave
        )));
    }

    gs.wave.clear_timer -= dt;
    if gs.wave.clear_timer <= 0.0 {
        gs.phase = Phase::Market;
        gs.market_timer = MARKET_DURATION;
        gs.market_sent = false;
        audio.play_market_open();
        let _ = event_tx.send(GameEvent::Broadcast("Market is open!".into()));
    }
}

fn start_next_wave(gs: &mut GameState, event_tx: &broadcast::Sender<GameEvent>, audio: &GameAudio) {
    let next = gs.wave.wave + 1;
    gs.wave.start(next, gs.screen_w);
    gs.phase = Phase::Playing;
    audio.play_wave_start();
    let _ = event_tx.send(GameEvent::Broadcast(format!("Wave {} — GO!", next)));
}

// ── Game over ────────────────────────────────────────────────────────────────

fn check_game_over(gs: &mut GameState, event_tx: &broadcast::Sender<GameEvent>, audio: &GameAudio) {
    if gs.players.is_empty() { return; }

    let mut is_game_over = false;

    let alive_players = gs.players.iter().filter(|p| p.alive).count();
    let respawning_players = gs.players.iter().filter(|p| p.is_respawning).count();
    let no_survivors = alive_players == 0 && respawning_players == 0;

    if no_survivors {
        is_game_over = true;
    } else if gs.is_convoy_mode() {
        if let Some(core) = &gs.convoy_core {
            if !core.alive {
                is_game_over = true;
            }
        }
    }

    if is_game_over {
        gs.last_summary = Some(gs.build_run_summary());
        gs.phase = Phase::GameOver;
        audio.play_game_over();
        let _ = event_tx.send(GameEvent::Broadcast("GAME OVER".into()));
    }
}

// ── Broadcast ────────────────────────────────────────────────────────────────

fn broadcast_player_states(gs: &GameState, event_tx: &broadcast::Sender<GameEvent>) {
    for p in &gs.players {
        let _ = event_tx.send(GameEvent::PlayerState {
            player_id: p.id,
            hp: p.hp,
            max_hp: p.max_hp,
            score: p.score,
            coins: p.coins,
            weapon_level: p.weapon_level,
            respawning: p.is_respawning,
            respawn_seconds: p.respawn_timer.max(0.0),
        });
    }
}

// ── Math helper ──────────────────────────────────────────────────────────────

#[inline]
fn circles_overlap(ax: f32, ay: f32, ar: f32, bx: f32, by: f32, br: f32) -> bool {
    let dx = ax - bx;
    let dy = ay - by;
    let sr = ar + br;
    dx * dx + dy * dy < sr * sr
}
