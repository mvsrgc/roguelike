use crate::{GodMode, Map, RunState, Viewshed, VisibilitySystem};

use super::{Player, Position, State, TileType, MAP_HEIGHT, MAP_WIDTH};
use rltk::{Rltk, VirtualKeyCode, Point};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let mut positions = gs.ecs.write_storage::<Position>();
    let mut players = gs.ecs.write_storage::<Player>();
    let mut viewsheds = gs.ecs.write_storage::<Viewshed>();
    let map = gs.ecs.fetch::<Map>();
    let godmode = gs.ecs.fetch::<GodMode>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let destination_index = map.map_index(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_index] != TileType::Wall || godmode.0 {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));

            let mut ppos = gs.ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn toggle_godmode(gs: &mut State) {
    let mut godmode = gs.ecs.fetch_mut::<GodMode>();

    if !godmode.0 {
        godmode.0 = true;
    } else {
        godmode.0 = false;
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::Paused, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, gs),
            VirtualKeyCode::Right => try_move_player(1, 0, gs),
            VirtualKeyCode::Up => try_move_player(0, -1, gs),
            VirtualKeyCode::Down => try_move_player(0, 1, gs),
            VirtualKeyCode::Key0 => toggle_godmode(gs),
            _ => return RunState::Paused,
        },
    }

    RunState::Running
}
