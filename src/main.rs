use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

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

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

pub struct State {
    ecs: World,
    god_mode: bool,
    revealed_tiles_before_godmode: Vec<bool>
}

impl State {
    fn run_systems(&mut self) {
        //lw.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
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

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }

        ctx.print(1, 1, format!("God: {}", self.god_mode));
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut game_state = State { ecs: World::new(), god_mode: false, revealed_tiles_before_godmode: vec![] };

    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<Viewshed>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    game_state.ecs.insert(map);

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
        .with(Viewshed{ visible_tiles : Vec::new(), range : 8 })
        .build();

    rltk::main_loop(context, game_state)
}
