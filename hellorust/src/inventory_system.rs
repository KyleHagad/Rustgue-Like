use specs::prelude::*;
use super::{Name, InBackpack, Position, gamelog::GameLog, CombatStats, Map,
            WantsToPickupItem, WantsToUseItem, WantsToDropItem, SufferDamage,
            Consumable, ProvidesHealing, InflictsDamage };

pub struct ItemCollectionSystem {}
impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>    );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut picksup, mut positions, names, mut backpack) = data;

        for pickup in picksup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by }).expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You found a {}.", names.get(pickup.item).unwrap().name));
            }
        }

        picksup.clear();
    }
}

pub struct ItemUseSystem {}
impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadExpect<'a, Map>,
                        ReadStorage<'a, Consumable>,
                        ReadStorage<'a, InflictsDamage>,
                        ReadStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>    );

    fn run(&mut self, data : Self::SystemData) {
        let (   player_entity, mut gamelog, entities, mut using_item, names, map,
                consumables, inflict_damage, healing, mut combat_stats, mut suffer_damage
            ) = data;

        for (entity, useitem, stats) in (&entities, &using_item, &mut combat_stats).join() {
            let mut used_item = true;

            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = useitem.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);
                    used_item = false;
                    for mob in map.tile_content[idx].iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!("You use the {} on the {}, inflicting {} damage.", item_name.name, mob_name.name, damage.damage));
                        }

                        used_item = true;
                    }
                }
            }

            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You eat the {}, restoring {} HP.", names.get(useitem.item).unwrap().name, healer.heal_amount));
                    }
                }
            }

            let consumable = consumables.get(useitem.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(useitem.item).expect("delete failed");
                }
            }
        }

        using_item.clear();
    }
}

pub struct ItemDropSystem {}
impl <'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>    );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{x:0,y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropper_pos.x, y : dropper_pos.y }).expect("Unable to insert");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}
