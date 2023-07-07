use rltk::{ RGB, Rltk, RandomNumberGenerator };
use super::{MAP_WIDTH, MAP_HEIGHT};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn new_map() -> Vec<TileType> {
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

