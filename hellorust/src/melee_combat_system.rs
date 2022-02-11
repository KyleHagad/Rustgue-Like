use specs::prelude::*;
use super::{ CombatStats, DoesMelee, Name, SufferDamage, GameLog };

pub struct MeleeCombatSystem {}
impl <'a> System<'a> for MeleeCombatSystem {
    type SystemData = ( Entities<'a>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, DoesMelee>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage> );

    fn run(&mut self, data : Self::SystemData) {
        let(entities, mut log, mut does_melee, names, combat_stats, mut inflict_damage) = data;

        for (_entity, does_melee, name, stats) in (&entities, &does_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(does_melee.target).unwrap();

                if target_stats.hp > 0 {
                    let target_name = names.get(does_melee.target).unwrap();
                    let damage = i32::max(0, stats.power - target_stats.defense);

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
