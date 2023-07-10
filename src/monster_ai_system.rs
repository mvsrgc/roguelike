use crate::{player, GodMode, Name};

use super::{Map, Monster, Position, Viewshed};
use rltk::{console, field_of_view, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        Read<'a, GodMode>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position, godmode) = data;

        for (mut viewshed, _monster, name, mut pos) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            if godmode.0 {
                return;
            }
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                console::log(&format!("{} shouts insults", name.name));
                return;
            }
            if viewshed.visible_tiles.contains(&*player_pos) && !godmode.0 {
                let path = rltk::a_star_search(
                    map.map_index(pos.x, pos.y),
                    map.map_index(player_pos.x, player_pos.y),
                    &mut *map,
                );

                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
