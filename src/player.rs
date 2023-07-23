use crate::{CombatStats, GodMode, Map, Name, RunState, Viewshed, WantsToMelee, GameLog, WantsToPickupItem, Item};

use super::{Player, Position, State, MAP_HEIGHT, MAP_WIDTH};
use rltk::{console, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let mut positions = gs.ecs.write_storage::<Position>();
    let mut players = gs.ecs.write_storage::<Player>();
    let mut viewsheds = gs.ecs.write_storage::<Viewshed>();
    let map = gs.ecs.fetch::<Map>();
    let godmode = gs.ecs.fetch::<GodMode>();
    let combat_stats = gs.ecs.read_storage::<CombatStats>();
    let name = gs.ecs.read_storage::<Name>();
    let entities = gs.ecs.entities();
    let mut wants_to_melee = gs.ecs.write_storage::<WantsToMelee>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }

        let destination_index = map.map_index(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_index].iter() {
            let target = combat_stats.get(*potential_target);
            let target_name = name.get(*potential_target);

            match (target, target_name) {
                (Some(_t), Some(target_name)) => {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed.");
                    console::log(format!("Player attacks {}", target_name.name));
                    return;
                }
                _ => {}
            }
        }

        if !map.blocked[destination_index] || godmode.0 {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));

            viewshed.dirty = true;

            let mut ppos = gs.ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;

            // Increment move count
            _player.number_of_moves += 1;
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
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::G => {
                get_item(&mut gs.ecs)
            }
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, gs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, gs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, gs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, gs)
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(-1, -1, gs),

            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(1, -1, gs),

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, gs),

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, gs),
            VirtualKeyCode::Key0 => toggle_godmode(gs),
            _ => return RunState::AwaitingInput,
        },
    }

    RunState::PlayerTurn
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pickup.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
}
