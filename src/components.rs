use std::collections::VecDeque;

use rltk::{RGB, Point, NavigationPath};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component)]
pub struct BlocksTile {}

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component)]
pub struct Monster {
    pub last_known_player_pos: Option<Point>,
    pub last_path: Option<NavigationPath>
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component, Debug)]
pub struct Player {
    pub number_of_moves: i32,
}
