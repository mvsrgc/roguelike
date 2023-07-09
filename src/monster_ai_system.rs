use crate::GodMode;

use super::{Map, Monster, Position, Viewshed};
use rltk::{console, field_of_view, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadExpect<'a, Point>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        ReadExpect<'a, GodMode>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_pos, viewshed, monster, godmode) = data;

        for (viewshed, _monster) in (&viewshed, &monster).join() {
            if viewshed.visible_tiles.contains(&*player_pos) && !godmode.0 {
                console::log("Monster shouts insult");
            }
        }
    }
}
