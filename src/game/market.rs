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
