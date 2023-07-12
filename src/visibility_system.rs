#![allow(unused)]
use crate::{GodMode, Player};
use specs::prelude::*;

use super::{Map, Position, Viewshed};
use rltk::{field_of_view, Point};

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        Read<'a, GodMode>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, godmode) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width as i32 && p.y >= 0 && p.y < map.height as i32);

                // If this is the player, reveal what they can see
                let _p: Option<&Player> = player.get(ent);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        if !godmode.0 {
                            // Don't add to revealed tiles while in godmode
                            let idx = map.map_index(vis.x, vis.y);
                            map.revealed_tiles[idx] = true;
                            map.visible_tiles[idx] = true;
                        }
                    }
                }
            }
        }

        if godmode.0 {
            map.visible_tiles.fill(true);
        }
    }
}
