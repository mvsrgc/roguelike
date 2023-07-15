use crate::{GameLog, Name, Player};

use super::{CombatStats, SufferDamage};

use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();

    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let entities = ecs.entities();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();

        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);

                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            let death_message = format!("{} is dead.", victim_name.name);
                            log.entries.push(death_message);
                        }
                        dead.push(entity);
                    }
                    Some(_) => {
                        log.entries.push("You are dead.".to_string());
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim)
            .expect("Unable to delete entity after death");
    }
}
