use std::cmp::{max, min};

use crate::{GodMode, Player, Rect, Viewshed};
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use specs::{Entity, Join, World, WorldExt};

use super::{MAP_HEIGHT, MAP_WIDTH};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: usize,
    pub height: usize,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width as i32;
        let y = idx as i32 / self.width as i32;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl Map {
    pub fn map_index(&self, x: i32, y: i32) -> usize {
        (y as usize * 80) + x as usize
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let index = self.map_index(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let index = self.map_index(x, y);
            if index > 0 && index < MAP_WIDTH as usize * MAP_HEIGHT as usize {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let index = self.map_index(x, y);
            if index > 0 && index < MAP_WIDTH as usize * MAP_HEIGHT as usize {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > (self.width - 1) as i32 || y < 1 || y > (self.height - 1) as i32 {
            return false;
        }
        let idx = self.map_index(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn new_map_rooms_and_corridors() -> Self {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAP_WIDTH as usize * MAP_HEIGHT as usize],
            revealed_tiles: vec![false; MAP_WIDTH as usize * MAP_HEIGHT as usize],
            visible_tiles: vec![false; MAP_WIDTH as usize * MAP_HEIGHT as usize],
            blocked: vec![false; MAP_WIDTH as usize * MAP_HEIGHT as usize],
            tile_content: vec![Vec::new(); MAP_WIDTH as usize * MAP_HEIGHT as usize],
            rooms: Vec::new(),
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
        };

        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: usize = 6;
        const MAX_SIZE: usize = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, (MAP_WIDTH - w - 1) as i32) - 1;
            let y = rng.roll_dice(1, (MAP_HEIGHT - h - 1) as i32) - 1;

            let new_room = Rect::new(x, y, w as i32, h as i32);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }

            if ok {
                map.apply_room_to_map(&new_room);

                // 50% chance to make a vertical tunnel first and then a
                // horizontal tunnel or vice-versa. This is because
                // if you have two squares that are distant from each other
                // you have two ways of connecting them (both ways form an L shape).
                // The randomness makes it less repetitive.
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                    map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
        let mut players = ecs.write_storage::<Player>();
        let mut viewsheds = ecs.write_storage::<Viewshed>();
        let map = ecs.fetch::<Map>();
        let godmode = ecs.fetch::<GodMode>();

        let mut x = 0;
        let mut y = 0;

        // Use enumerate to get the index of each tile,
        // the tile index allows us to find if that tile
        // is True in map.revealed_tiles (since map.revealed_tiles[0]
        // maps to map.tiles[0])
        for (index, tile) in map.tiles.iter().enumerate() {
            if map.revealed_tiles[index] || godmode.0 {
                let glyph;
                let mut fg;
                match tile {
                    TileType::Floor => {
                        fg = RGB::from_f32(0.5, 0.5, 0.5);
                        glyph = rltk::to_cp437('.');
                    }
                    TileType::Wall => {
                        glyph = rltk::to_cp437('#');
                        fg = RGB::from_f32(0., 1.0, 0.);
                    }
                }
                if !map.visible_tiles[index] {
                    fg = fg.to_greyscale()
                }
                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
            }
            x += 1;
            if x > MAP_WIDTH - 1 {
                x = 0;
                y += 1;
            }
        }
    }
}
