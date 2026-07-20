use crate::server::MarketItem;
use super::state::Player;

/// สร้าง list ของ upgrades ที่ผู้เล่นสามารถซื้อได้
pub fn get_offers(p: &Player) -> Vec<MarketItem> {
    let mut offers = Vec::new();

    if p.weapon_level < 5 {
        let cost = weapon_cost(p.weapon_level);
        offers.push(MarketItem {
            id: "weapon_up".into(),
            name: format!("Weapon Lv{}", p.weapon_level + 1),
            description: format!("Upgrade weapon spread  (cost: {} coins)", cost),
            cost,
        });
    }

    if p.hp < p.max_hp {
        offers.push(MarketItem {
            id: "hp_restore".into(),
            name: "Repair Ship".into(),
            description: format!("Restore HP to full {}/{} (cost: 30)", p.max_hp, p.max_hp),
            cost: 30,
        });
    }

    offers.push(MarketItem {
        id: "max_hp_up".into(),
        name: "+1 Max HP".into(),
        description: format!("Max HP {} → {} (cost: 60)", p.max_hp, p.max_hp + 1),
        cost: 60,
    });

    offers
}

/// ลองซื้อ item ด้วย item_id; คืน true ถ้าซื้อสำเร็จ
pub fn apply_purchase(p: &mut Player, item_id: &str) -> bool {
    match item_id {
        "weapon_up" => {
            let cost = weapon_cost(p.weapon_level);
            if p.weapon_level < 5 && p.coins >= cost {
                p.coins -= cost;
                p.weapon_level += 1;
                true
            } else {
                false
            }
        }
        "hp_restore" => {
            if p.hp < p.max_hp && p.coins >= 30 {
                p.coins -= 30;
                p.hp = p.max_hp;
                true
            } else {
                false
            }
        }
        "max_hp_up" => {
            if p.coins >= 60 {
                p.coins -= 60;
                p.max_hp += 1;
                p.hp = (p.hp + 1).min(p.max_hp);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn weapon_cost(current_level: u8) -> u32 {
    match current_level {
        1 => 40,
        2 => 70,
        3 => 110,
        4 => 160,
        _ => 999,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// สร้าง player ที่กำหนดค่าที่ส่งผลต่อ market ได้โดยตรง
    fn player_with(coins: u32, weapon_level: u8, hp: u8, max_hp: u8) -> Player {
        let mut p = Player::new(0, 0.0, 0.0);
        p.coins = coins;
        p.weapon_level = weapon_level;
        p.hp = hp;
        p.max_hp = max_hp;
        p
    }

    // ── apply_purchase: weapon_up ──────────────────────────────────────────────

    #[test]
    fn weapon_up_success_at_level_1_costs_40() {
        let mut p = player_with(40, 1, 3, 3);
        assert!(apply_purchase(&mut p, "weapon_up"));
        assert_eq!(p.weapon_level, 2);
        assert_eq!(p.coins, 0);
    }

    #[test]
    fn weapon_up_cost_scales_with_level() {
        // ราคา 40 / 70 / 110 / 160 ตาม level ปัจจุบัน
        for (level, cost) in [(1u8, 40u32), (2, 70), (3, 110), (4, 160)] {
            let mut p = player_with(cost, level, 3, 3);
            assert!(
                apply_purchase(&mut p, "weapon_up"),
                "weapon_up ควรสำเร็จที่ level {level} ด้วย {cost} coins"
            );
            assert_eq!(p.weapon_level, level + 1);
            assert_eq!(p.coins, 0);
        }
    }

    #[test]
    fn weapon_up_capped_at_level_5() {
        let mut p = player_with(999, 5, 3, 3);
        assert!(!apply_purchase(&mut p, "weapon_up"));
        // ไม่เปลี่ยนแปลงอะไร
        assert_eq!(p.weapon_level, 5);
        assert_eq!(p.coins, 999);
    }

    #[test]
    fn weapon_up_rejects_insufficient_coins() {
        let mut p = player_with(39, 1, 3, 3);
        assert!(!apply_purchase(&mut p, "weapon_up"));
        assert_eq!(p.weapon_level, 1);
        assert_eq!(p.coins, 39);
    }

    #[test]
    fn weapon_up_at_max_level_does_not_drain_coins_even_if_rich() {
        let mut p = player_with(999, 5, 3, 3);
        assert!(!apply_purchase(&mut p, "weapon_up"));
        assert_eq!(p.coins, 999, "ต้องไม่หัก coins เมื่อ reject");
    }

    // ── apply_purchase: hp_restore ─────────────────────────────────────────────

    #[test]
    fn hp_restore_refills_hp_for_30_coins() {
        let mut p = player_with(30, 1, 1, 3);
        assert!(apply_purchase(&mut p, "hp_restore"));
        assert_eq!(p.hp, 3);
        assert_eq!(p.coins, 0);
    }

    #[test]
    fn hp_restore_rejects_when_already_full() {
        let mut p = player_with(30, 1, 3, 3);
        assert!(!apply_purchase(&mut p, "hp_restore"));
        assert_eq!(p.hp, 3);
        assert_eq!(p.coins, 30, "ต้องไม่หัก coins เมื่อ reject");
    }

    #[test]
    fn hp_restore_rejects_insufficient_coins() {
        let mut p = player_with(29, 1, 1, 3);
        assert!(!apply_purchase(&mut p, "hp_restore"));
        assert_eq!(p.hp, 1);
    }

    // ── apply_purchase: max_hp_up ──────────────────────────────────────────────

    #[test]
    fn max_hp_up_increases_both_max_and_current_for_60_coins() {
        let mut p = player_with(60, 1, 3, 3);
        assert!(apply_purchase(&mut p, "max_hp_up"));
        assert_eq!(p.max_hp, 4);
        assert_eq!(p.hp, 4);
        assert_eq!(p.coins, 0);
    }

    #[test]
    fn max_hp_up_keeps_hp_proportional_when_damaged() {
        // hp ปัจจุบันตามมา +1 แต่ไม่เกิน max_hp ใหม่
        let mut p = player_with(60, 1, 2, 3);
        assert!(apply_purchase(&mut p, "max_hp_up"));
        assert_eq!(p.max_hp, 4);
        assert_eq!(p.hp, 3, "hp = (2+1).min(4) = 3");
    }

    #[test]
    fn max_hp_up_rejects_insufficient_coins() {
        let mut p = player_with(59, 1, 3, 3);
        assert!(!apply_purchase(&mut p, "max_hp_up"));
        assert_eq!(p.max_hp, 3);
        assert_eq!(p.coins, 59);
    }

    // ── apply_purchase: unknown ids ────────────────────────────────────────────

    #[test]
    fn unknown_item_id_is_rejected_without_side_effects() {
        let mut p = player_with(100, 1, 3, 3);
        assert!(!apply_purchase(&mut p, "foo"));
        assert_eq!(p.coins, 100);
        assert_eq!(p.weapon_level, 1);
        assert_eq!(p.hp, 3);
        assert_eq!(p.max_hp, 3);
    }

    // ── get_offers ─────────────────────────────────────────────────────────────

    #[test]
    fn fresh_full_hp_player_offers_weapon_up_and_max_hp_up_only() {
        let p = player_with(100, 1, 3, 3);
        let offers = get_offers(&p);
        let ids: Vec<&str> = offers.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(ids, ["weapon_up", "max_hp_up"]);
    }

    #[test]
    fn damaged_player_also_gets_hp_restore() {
        let p = player_with(100, 1, 1, 3);
        let offers = get_offers(&p);
        let ids: Vec<&str> = offers.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(ids, ["weapon_up", "hp_restore", "max_hp_up"]);
    }

    #[test]
    fn max_level_player_does_not_get_weapon_up() {
        let p = player_with(100, 5, 3, 3);
        let offers = get_offers(&p);
        let ids: Vec<&str> = offers.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(ids, ["max_hp_up"]);
    }

    #[test]
    fn weapon_up_offer_cost_matches_cost_table() {
        for (level, expected_cost) in [(1u8, 40u32), (2, 70), (3, 110), (4, 160)] {
            let p = player_with(999, level, 3, 3);
            let offers = get_offers(&p);
            let weapon_offer = offers.iter().find(|o| o.id == "weapon_up").unwrap();
            assert_eq!(
                weapon_offer.cost, expected_cost,
                "ราคา weapon_up ที่ level {level} ต้องเป็น {expected_cost}"
            );
        }
    }

    #[test]
    fn fixed_costs_match_contract() {
        let p = player_with(999, 1, 1, 3);
        let offers = get_offers(&p);
        let by_id: std::collections::HashMap<&str, u32> =
            offers.iter().map(|o| (o.id.as_str(), o.cost)).collect();
        assert_eq!(by_id["hp_restore"], 30);
        assert_eq!(by_id["max_hp_up"], 60);
    }
}
