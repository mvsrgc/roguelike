use crate::{Map, Viewshed};

use super::{Player, Position, State, TileType, MAP_HEIGHT, MAP_WIDTH};
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let mut positions = gs.ecs.write_storage::<Position>();
    let mut players = gs.ecs.write_storage::<Player>();
    let mut viewsheds = gs.ecs.write_storage::<Viewshed>();
    let map = gs.ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_index = map.map_index(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_index] != TileType::Wall || gs.god_mode {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));

            viewshed.dirty = true;
        }
    }
}

pub fn toggle_godmode(gs: &mut State) {
    let mut map = gs.ecs.fetch_mut::<Map>();

    if !gs.god_mode {
        gs.revealed_tiles_before_godmode = map.revealed_tiles.clone();
        map.revealed_tiles.fill(true);
        gs.god_mode = true;
    } else {
        gs.god_mode = false;
        map.revealed_tiles = gs.revealed_tiles_before_godmode.clone();
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, gs),
            VirtualKeyCode::Right => try_move_player(1, 0, gs),
            VirtualKeyCode::Up => try_move_player(0, -1, gs),
            VirtualKeyCode::Down => try_move_player(0, 1, gs),
            VirtualKeyCode::Key0 => toggle_godmode(gs),
            _ => {}
        },
    }
}
