use ggez::Context;
use gwg as ggez;

use crate::entities::teleporter::{Exit, Teleporter};
use crate::entities::{cloud, foilage, player, skeleton};
use crate::sprite::SpriteComponent;
use crate::states::game_state::GameState;
use crate::transform_compontent::TransformComponent;
use crate::util;
use foilage::Grass;
use nalgebra as na;
use player::Player;
use skeleton::SkeletonBlock;
use std::io::Read;

const MAP_NAMES: &[&str] = &[
    "/maps/map_first.txt",
    "/maps/map_1skeleton.txt",
    "/maps/map_2skeleton.txt",
    "/maps/map_gravity.txt",
    "/maps/map_teleport.txt",
    "/maps/map_simple_backtrack.txt",
    "/maps/map_follow.txt",
    "/maps/map_middleclash.txt",
    "/maps/map_2skeleton_intro.txt",
    "/maps/map_2skeleton_backtrack.txt",
    "/maps/map_force_stand.txt",
    "maps/map_fall_trap.txt",
    "/maps/map_skeleton_hole.txt",
    "/maps/map_easy_accidental_fall.txt",
    "/maps/map_easy3.txt",
    "/maps/map_easy1.txt",
    "/maps/map_maze1.txt",
    "/maps/map_follow_2.txt",
    "/maps/map_easy2.txt",
    "/maps/map_3skeleton.txt",
    "/maps/map_3skeleton_3holes.txt",
    "/maps/map_skeleton_platform.txt",
    "/maps/map_middleclash_2.txt",
    "/maps/map_3skeleton_3holes_harder.txt",
    "/maps/map_hard1.txt",
    "/maps/map_hard3.txt",
    "/maps/map_hard2.txt",
];

pub const MAP_COUNT: usize = MAP_NAMES.len();

fn get_map_name(index: usize) -> &'static str {
    MAP_NAMES[index]
}

pub fn clear_map(game_state: &mut GameState) {
    game_state.grasses.clear();
    game_state.skeletons.clear();
    game_state.skeleton_blocks.clear();
    game_state.foilages.clear();
    game_state.clouds.clear();
    game_state.teleporters[0] = None;
    game_state.teleporters[1] = None;
}

pub fn load_map(
    ctx: &mut Context,
    game_state: &mut GameState,
    map_index: usize,
    screen_size: &na::Point2<f32>,
) {
    let map_filename = get_map_name(map_index);
    let mut file = ggez::filesystem::open(ctx, map_filename).expect("no map file");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    for char in buffer.chars() {
        // almost every case have a transform, create here to avoid redundant code
        let transform = TransformComponent {
            position: na::Point2::new(x, y),
        };

        match char {
            '0' => {
                game_state.player = Player {
                    transform,
                    is_alive: true,
                    ..Default::default()
                };
            }
            '1' => {
                game_state.grasses.push(Grass {
                    transform,
                    sprite: SpriteComponent {
                        texture_index: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            '2' => {
                game_state.skeleton_blocks.push(SkeletonBlock {
                    transform,
                    sprite: SpriteComponent {
                        texture_index: 3,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            '3' => {
                let sprite = SpriteComponent {
                    texture_index: 5,
                    ..Default::default()
                };
                if game_state.teleporters[0].is_none() {
                    game_state.teleporters[0] = Some(Teleporter {
                        transform,
                        sprite,
                        ..Default::default()
                    });
                } else {
                    game_state.teleporters[1] = Some(Teleporter {
                        transform,
                        sprite,
                        ..Default::default()
                    });
                }
            }
            '4' => {
                game_state.exit = Exit {
                    transform,
                    sprite: SpriteComponent {
                        texture_index: 6,
                        ..Default::default()
                    },
                    ..Default::default()
                };
            }
            '5' => {
                game_state.grasses.push(Grass {
                    transform,
                    sprite: SpriteComponent {
                        texture_index: 10,
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            '\n' => {
                // -1 because it will increment right after to 0
                x = -1;
                y += 1;
            }
            _ => {}
        }
        x += 1;
    }
    // visual position starts at 0,0
    util::force_visual_positions(game_state, screen_size);

    // Clouds generation
    cloud::spawn(game_state);

    foilage::generate(game_state);
}
