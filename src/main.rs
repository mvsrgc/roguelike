use rltk::{GameState, Rltk, Tile, VirtualKeyCode, RGB};
use specs::{hibitset::BitSetLike, prelude::*};
use specs_derive::Component;
use std::cmp::{max, min};

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

struct State {
    ecs: World,
}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

fn new_map() -> Vec<TileType> {
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

fn draw_map(map: &[TileType], ctx: &mut Rltk) {
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

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_index = map_index(pos.x + delta_x, pos.y + delta_y);
        if map[destination_index] != TileType::Wall {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }

        ctx.print(1, 1, "Hello Roguelike!");
    }
}

impl State {
    fn run_systems(&mut self) {
        //lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut game_state = State { ecs: World::new() };

    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.insert(new_map());

    game_state
        .ecs
        .create_entity()
        .with(Position { x: 30, y: 30 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, game_state)
}
