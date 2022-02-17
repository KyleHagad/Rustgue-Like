use specs::prelude::*;
use super::{
    ThirstClock, RunState, ThirstState, SufferDamage, gamelog::GameLog,
};

pub struct ThirstSystem { }
impl<'a> System<'a> for ThirstSystem {
    type SystemData = ( Entities<'a>,
                        WriteStorage<'a, ThirstClock>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        WriteStorage<'a, SufferDamage>,
                        WriteExpect<'a, GameLog>, );

    fn run(&mut self, data : Self::SystemData) {
        let (
            entities,
            mut thirst_clock,
            player_entity,
            runstate,
            mut inflict_damage,
            mut log,
        ) = data;

        for (entity, mut clock) in (&entities, &mut thirst_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => {
                    if entity == *player_entity { proceed = true; }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity { proceed = true; }
                }
                _ => proceed = false
            }

            if proceed {
                clock.duration -= 1;
                if clock.duration < 1 {
                    match clock.state {
                        ThirstState::Quenched => {
                            clock.state = ThirstState::Normal;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are no longer quenched.".to_string());
                            }
                        }
                        ThirstState::Normal => {
                            clock.state = ThirstState::Thirsty;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are thirsty.".to_string());
                            }
                        }
                        ThirstState::Thirsty => {
                            clock.state = ThirstState::Parched;
                            clock.duration = 200;
                            if entity == *player_entity {
                                log.entries.push("You are dangerously dehydrated.".to_string());
                            }
                        }
                        ThirstState::Parched => {
                            if entity == *player_entity {
                                log.entries.push("Your thirst hurts.".to_string());
                            }
                            SufferDamage::new_damage(&mut inflict_damage, entity, 2);
                        }
                    }
                }
            }
        }
    }
}
