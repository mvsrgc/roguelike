use crate::GameLog;

use super::{CombatStats, Name, SufferDamage, WantsToMelee};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_to_melee, names, combat_stats, mut inflict_damage, mut gamelog) =
            data;

        for (_entity, wants_to_melee, name, stats) in
            (&entities, &wants_to_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = match combat_stats.get(wants_to_melee.target) {
                    Some(target_stats) => target_stats,
                    None => return,
                };

                if target_stats.hp > 0 {
                    let target_name = names.get(wants_to_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

                    if damage == 0 {
                        let unable_hit_message =
                            format!("{} is unable to hurt {}", &name.name, &target_name.name);
                        gamelog.entries.push(unable_hit_message);
                    } else {
                        let hit_message = format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        );
                        gamelog.entries.push(hit_message);
                        SufferDamage::new_damage(&mut inflict_damage, wants_to_melee.target, damage)
                    }
                }
            }
        }

        wants_to_melee.clear();
    }
}
