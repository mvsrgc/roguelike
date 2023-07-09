use rltk::{GameState, RandomNumberGenerator, Rltk, RGB};
use specs::prelude::*;

mod monster_ai_system;
pub use monster_ai_system::*;

mod player;
pub use player::*;

mod map;
pub use map::*;

mod components;
pub use components::*;

mod rect;
pub use rect::*;

mod visibility_system;
pub use visibility_system::*;

#[derive(Default)]
pub struct GodMode(bool);

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        //lw.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        Map::draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let index = map.map_index(pos.x, pos.y);
            if map.visible_tiles[index] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }

        let godmode = self.ecs.fetch::<GodMode>();
        ctx.print(1, 1, format!("God: {}", godmode.0));
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
    game_state.ecs.register::<Viewshed>();
    game_state.ecs.register::<Monster>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut rng = RandomNumberGenerator::new();
    for room in map.rooms.iter().skip(1) {
        let (x, y) = room.center();

        let glyph: rltk::FontCharType;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => glyph = rltk::to_cp437('g'),
            _ => glyph = rltk::to_cp437('o'),
        }
        game_state
            .ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .build();
    }

    game_state.ecs.insert(map);

    game_state.ecs.insert(GodMode(false));

    game_state
        .ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();

    rltk::main_loop(context, game_state)
}
