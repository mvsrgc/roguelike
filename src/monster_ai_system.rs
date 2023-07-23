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
            }

            // Monster sees the player, he remembers the location of the player
            if viewshed.visible_tiles.contains(&*player_pos) {
                _monster.last_known_player_pos = Some(*player_pos);
            }

            // TODO: Monsters close to each other should share a path
            // instead of recalculating.
            if let Some(last_known_player_pos) = _monster.last_known_player_pos {
                if _monster.last_pathfind.is_none()
                    || _monster.last_pathfind.as_mut().unwrap().steps.len() <= 1
                    || viewshed.visible_tiles.contains(&*player_pos)
                {
                    _monster.last_pathfind = Some(rltk::a_star_search(
                        map.map_index(pos.x, pos.y) as i32,
                        map.map_index(last_known_player_pos.x, last_known_player_pos.y) as i32,
                        &mut *map,
                    ));
                }

                let path = _monster.last_pathfind.as_mut().unwrap();
                if path.success && path.steps.len() > 1 {
                    // Monster will move, so his current location will not be blocked anymore.
                    let mut idx = map.map_index(pos.x, pos.y);
                    map.blocked[idx] = false;

                    // New position which is calculated by A*
                    let step = path.steps.remove(1);
                    pos.x = step as i32 % map.width;
                    pos.y = step as i32 / map.width;

                    idx = map.map_index(pos.x, pos.y);

                    // The new location the monster moved to is blocked.
                    map.blocked[idx] = true;

                    viewshed.dirty = true;
                    _monster.last_pathfind = Some(path.clone());
                } else {
                    _monster.last_pathfind = None;
                }
            }
        }
    }
}
