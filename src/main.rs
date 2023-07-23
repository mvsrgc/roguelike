use rltk::{GameState, Point, RandomNumberGenerator, Rltk};
use specs::prelude::*;

mod inventory_system;
pub use inventory_system::*;

mod spawner;
pub use spawner::*;

mod gamelog;
pub use gamelog::*;

mod gui;
pub use gui::*;

mod damage_system;

pub use damage_system::*;

mod melee_combat_system;
pub use melee_combat_system::*;

mod map_indexing_system;
pub use map_indexing_system::*;

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

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 43;
pub const MAP_COUNT: i32 = MAP_WIDTH * MAP_HEIGHT;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

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

        let godmode: specs::shred::Fetch<GodMode> = self.ecs.fetch();
        ctx.print(1, 1, format!("God: {}", godmode.0));
        ctx.print(1, 2, format!("FPS: {}", ctx.fps));

        draw_ui(&self.ecs, ctx);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("")
        .with_fps_cap(120.0)
        .build()?;
    context.with_post_scanlines(true);
    let mut game_state = State { ecs: World::new() };

    game_state.ecs.register::<Item>();
    game_state.ecs.register::<Potion>();
    game_state.ecs.register::<SufferDamage>();
    game_state.ecs.register::<WantsToMelee>();
    game_state.ecs.register::<CombatStats>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<Viewshed>();
    game_state.ecs.register::<Monster>();
    game_state.ecs.register::<Name>();
    game_state.ecs.register::<BlocksTile>();
    game_state.ecs.register::<WantsToPickupItem>();
    game_state.ecs.register::<InInventory>();
    game_state.ecs.register::<LastPathUpdate>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = spawner::player(&mut game_state.ecs, player_x, player_y);
    game_state.ecs.insert(player_entity);

    game_state.ecs.insert(RandomNumberGenerator::new());

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut game_state.ecs, room);
    }

    game_state.ecs.insert(map);
    game_state.ecs.insert(GodMode(false));
    game_state.ecs.insert(Point::new(player_x, player_y));
    game_state.ecs.insert(RunState::PreRun);
    game_state.ecs.insert(GameLog {
        entries: vec!["Hello, sailor!".to_string()],
    });

    rltk::main_loop(context, game_state)
}
