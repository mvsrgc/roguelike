use crate::{Name, RunState, WantsToMelee};

use super::{Map, Monster, Position, Viewshed};
use rltk::{console, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            mut monster,
            name,
            mut position,
            mut wants_to_melee,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _monster, name, pos) in
            (&entities, &mut viewshed, &mut monster, &name, &mut position).join()
        {
            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);

            // Monster is close, he can attack
            if distance < 1.5 {
                console::log(&format!("{} shouts insults", name.name));
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *player_entity,
                        },
                    )
                    .expect("Unable to insert attack.");
                _monster.last_known_player_pos = Some(*player_pos);
                continue;
            } else if distance >= 8.0 {
                _monster.last_known_player_pos = None;
                continue;
            }

            // Monster sees the player, he remembers the location of the player
            if viewshed.visible_tiles.contains(&*player_pos) {
                _monster.last_known_player_pos = Some(*player_pos);
            }

            // Monster remembers a location, find a path to it
            // @Cleanup Make the monster remember a path so we don't recalculate
            // unnecessarily.
            if let Some(last_known_player_pos) = _monster.last_known_player_pos {
                let path = rltk::a_star_search(
                    map.map_index(pos.x, pos.y) as i32,
                    map.map_index(last_known_player_pos.x, last_known_player_pos.y) as i32,
                    &mut *map,
                );

                if path.success && path.steps.len() > 1 {
                    // Monster will move, so his current location will not be blocked anymore.
                    let mut idx = map.map_index(pos.x, pos.y);
                    map.blocked[idx] = false;

                    // New position which is calculated by A*
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;

                    idx = map.map_index(pos.x, pos.y);

                    // The new location the monster moved to is blocked.
                    map.blocked[idx] = true;

                    viewshed.dirty = true;
                }
            }
        }
    }
}
