use std::cmp::{max, min};

use crate::Rect;
use rltk::{RandomNumberGenerator, Rltk, RGB};

use super::{MAP_HEIGHT, MAP_WIDTH};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[map_index(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let index = map_index(x, y);
        if index > 0 && index < MAP_WIDTH as usize * MAP_HEIGHT as usize {
            map[index as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let index = map_index(x, y);
        if index > 0 && index < MAP_WIDTH as usize * MAP_HEIGHT as usize {
            map[index as usize] = TileType::Floor;
        }
    }
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let dimensions: usize = MAP_WIDTH as usize * MAP_HEIGHT as usize;
    let mut map = vec![TileType::Wall; dimensions];

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, MAP_WIDTH - w - 1) - 1;
        let y = rng.roll_dice(1, MAP_HEIGHT - h - 1) - 1;

        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in rooms.iter() {
            if new_room.intersect(other_room) {
                ok = false
            }
        }

        if ok {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, new_y);
                }
            }

            rooms.push(new_room);
        }
    }

    (rooms, map)
}

pub fn new_map_test() -> Vec<TileType> {
    let dimensions: usize = MAP_WIDTH as usize * MAP_HEIGHT as usize;
    let mut map = vec![TileType::Floor; dimensions];

    for x in 0..MAP_WIDTH {
        map[map_index(x, 0)] = TileType::Wall;
        map[map_index(x, MAP_HEIGHT - 1)] = TileType::Wall;
    }

    for y in 0..MAP_HEIGHT {
        map[map_index(0, y)] = TileType::Wall;
        map[map_index(MAP_WIDTH - 1, y)] = TileType::Wall;
    }

    // Randomly put some walls everywhere
    let mut rng = rltk::RandomNumberGenerator::new();
    for _i in 0..200 {
        let x = rng.roll_dice(1, MAP_WIDTH - 1);
        let y = rng.roll_dice(1, MAP_HEIGHT - 1);
        let index = map_index(x, y);

        // Don't put a wall in the exact middle, where the player starts
        if index != map_index(40, 25) {
            map[index] = TileType::Wall;
        }
    }
    map
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut x = 0;
    let mut y = 0;

    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }
        x += 1;
        if x > MAP_WIDTH - 1 {
            x = 0;
            y += 1;
        }
    }
}

pub fn map_index(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}
