use specs::prelude::*;
use super::{ CombatStats, DoesMelee, Name, SufferDamage, GameLog, MeleePowerBonus, DefenseBonus, Equipped };

pub struct MeleeCombatSystem {}
impl <'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, DoesMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, MeleePowerBonus>,
                        ReadStorage<'a, DefenseBonus>,
                        ReadStorage<'a, Equipped>   );

    fn run(&mut self, data : Self::SystemData) {
        let (
            entities,
            mut log,
            mut does_melee,
            names,
            combat_stats,
            mut inflict_damage,
            melee_power_bonus,
            defense_bonus,
            equipped,
        ) = data;

        for (entity, does_melee, name, stats) in (&entities, &does_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in (&entities, &melee_power_bonus, &equipped).join() {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let target_stats = combat_stats.get(does_melee.target).unwrap();

                if target_stats.hp > 0 {
                    let target_name = names.get(does_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in (&entities, &defense_bonus, &equipped).join() {
                        if equipped_by.owner == does_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(0, (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus));

                    if damage == 0 {
                        log.entries.push(format!("{} cannot be touched by {}", &target_name.name, &name.name));
                    } else {
                        log.entries.push(format!("{} hugs {} for {} seconds", &name.name, &target_name.name, damage));
                        SufferDamage::new_damage(&mut inflict_damage, does_melee.target, damage);
                    }
                }
            }
        }

        does_melee.clear();
    }
}
