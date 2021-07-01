use crate::constantes;
use crate::entities::{
    foilage::Grass,
    skeleton::{Skeleton, SkeletonBlock},
    ai::AiState
};
use crate::particle_system;
use crate::particle_system::ParticleSystemCollection;
use crate::sound_collection::SoundCollection;
use crate::sprite::SpriteComponent;
use crate::{map, states::game_state::GameState, transform_compontent::TransformComponent};
use gwg::Context;
use nalgebra as na;
#[derive(PartialEq)]
pub enum PlayerInputIntent {
    None,
    Up,
    Left,
    Right,
    Down,
}

impl Default for PlayerInputIntent {
    fn default() -> Self {
        PlayerInputIntent::None
    }
}

//#[derive(Default)]
pub struct Player {
    pub transform: TransformComponent,
    pub sprite: SpriteComponent,
    pub input_intent: PlayerInputIntent,
    pub time_since_step: f32,
    pub is_alive: bool,
    pub is_on_skeleton: bool, // Used for other to look at
    pub prev_grounded: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            prev_grounded: true,
            is_on_skeleton: false,
            is_alive: true,
            time_since_step: 0.0,
            input_intent: PlayerInputIntent::None,
            sprite: SpriteComponent::default(),
            transform: TransformComponent::default(),
        }
    }
}

impl Player {
    pub fn should_step(
        &mut self,
        dt: f32,
        grasses: &Vec<Grass>,
        skeletons: &Vec<Skeleton>,
        skeleton_blocks: &Vec<SkeletonBlock>,
        sound_collection: &mut SoundCollection,
        particle_collection: &mut ParticleSystemCollection,
        land_id: &u32,
        screen_size: &na::Point2<f32>,
    ) -> bool {
        if !self.is_alive {
            return false;
        }

        self.time_since_step += dt;
        let pos_below = self.transform.position + na::Vector2::new(0, 1);
        let mut is_grounded = skeletons.iter().any(|s| s.transform.position == pos_below);
        self.is_on_skeleton = is_grounded;
        is_grounded |= grasses.iter().any(|g| g.transform.position == pos_below);
        is_grounded |= skeleton_blocks
            .iter()
            .any(|s| s.transform.position == pos_below);

        if self.input_intent != PlayerInputIntent::None && is_grounded {
            self.time_since_step = 0.0;
            return true;
        }

        if self.time_since_step > constantes::TIME_AUTO_STEP {
            if !is_grounded {
                self.time_since_step = 0.0;
                return true;
            }

            if is_grounded && !self.prev_grounded {
                self.sprite.texture_index = 0;
                self.prev_grounded = true;
                sound_collection.play(8);

                let mut land_particles = particle_collection.get_mut(*land_id).unwrap();
                land_particles.scale = screen_size.x / 16.0;
                let pos_player_visual = self.sprite.visual_position;
                let mut pos_particle = na::Vector2::new(
                    pos_player_visual.x / screen_size.x * 16.0,
                    pos_player_visual.y / screen_size.x * 16.0,
                );
                pos_particle += na::Vector2::new(16.0 * 0.5, 16.0);

                land_particles.position = pos_particle;
                land_particles.emit(15);
            }
        }
        false
    }
}

pub fn system(
    game_state: &mut GameState,
    ctx: &mut Context,
    current_map: &mut usize,
    sound_collection: &mut SoundCollection,
    screen_size: &na::Point2<f32>,
    particle_collection: &mut ParticleSystemCollection,
    grass_id: &u32,
    step_id: &u32,
    foilage_1_id: &u32,
    foilage_2_id: &u32,
    foilage_3_id: &u32,
    foilage_4_id: &u32,
) {
    let mut should_exit = false;

    let player = &mut game_state.player;
    let pos_below = player.transform.position + na::Vector2::new(0, 1);
    let mut is_grounded = game_state
        .grasses
        .iter()
        .any(|g| g.transform.position == pos_below);
    is_grounded |= game_state
        .skeletons
        .iter()
        .any(|s| s.transform.position == pos_below);
    is_grounded |= game_state
        .skeleton_blocks
        .iter()
        .any(|s| s.transform.position == pos_below);

    if player.prev_grounded && !is_grounded {
        sound_collection.play(7);
        player.sprite.texture_index = 13;
    }

    player.prev_grounded = is_grounded;
    if !is_grounded {
        player.transform.position = pos_below;

        if player.transform.position.y > constantes::GAME_BOUNDS_Y {
            player.transform.position.y = 0;
            // Force visual insta jump
            let pos_player_unscaled =
                na::convert::<na::Point2<i32>, na::Point2<f32>>(player.transform.position);
            player.sprite.visual_position = pos_player_unscaled * screen_size.x;
            player.sprite.blink_timer = constantes::TIME_BLINK;
        }
        return;
    }
    player.sprite.texture_index = 0;

    match player.input_intent {
        PlayerInputIntent::Left => {
            player.sprite.is_flipped = true;
            let new_position = player.transform.position - na::Vector2::new(1, 0);
            let occupied_by_grass = game_state
                .grasses
                .iter()
                .any(|g| g.transform.position == new_position);
            let occupied_by_skeleton = game_state
                .skeletons
                .iter()
                .any(|s| s.transform.position == new_position);
            let occupied_by_skeleton_block = game_state
                .skeleton_blocks
                .iter()
                .any(|s| s.transform.position == new_position);
            let is_occupied =
                occupied_by_grass || occupied_by_skeleton || occupied_by_skeleton_block;
            if !is_occupied {
                player.transform.position = new_position;
                sound_collection.play(0);

                particle_system::emit_step_particle(
                    particle_collection,
                    step_id,
                    9,
                    false,
                    &player.sprite.visual_position,
                    screen_size,
                );
            }
        }
        PlayerInputIntent::Right => {
            player.sprite.is_flipped = false;
            let new_position = player.transform.position + na::Vector2::new(1, 0);
            let occupied_by_grass = game_state
                .grasses
                .iter()
                .any(|g| g.transform.position == new_position);
            let occupied_by_skeleton = game_state
                .skeletons
                .iter()
                .any(|s| s.transform.position == new_position);
            let occupied_by_skeleton_block = game_state
                .skeleton_blocks
                .iter()
                .any(|s| s.transform.position == new_position);
            let is_occupied =
                occupied_by_grass || occupied_by_skeleton || occupied_by_skeleton_block;
            if !is_occupied {
                player.transform.position = new_position;
                sound_collection.play(0);
                particle_system::emit_step_particle(
                    particle_collection,
                    step_id,
                    8,
                    true,
                    &player.sprite.visual_position,
                    screen_size,
                );
            }
        }
        PlayerInputIntent::Up => {
            // Teleporter
            let teleporter_tuple_option = game_state
                .teleporters
                .iter()
                .map(|t| t.as_ref())
                .enumerate()
                .find(|(_i, t)| {
                    if let Some(teleporter) = t {
                        return teleporter.transform.position == player.transform.position;
                    }
                    false
                });
            if let Some((index, _teleporter)) = teleporter_tuple_option {
                let other_teleporter_index = 1 - index;
                let other_teleporter_option = game_state
                    .teleporters
                    .get_mut(other_teleporter_index)
                    .unwrap();
                if let Some(other_teleporter) = other_teleporter_option {
                    // Force visual insta jump
                    player.transform.position = other_teleporter.transform.position;
                    player.sprite.visual_position = other_teleporter.sprite.visual_position;
                    sound_collection.play(3);
                    player.sprite.blink_timer = constantes::TIME_BLINK;
                    other_teleporter.sprite.blink_timer = constantes::TIME_BLINK;

                    let skeleton_option = game_state
                        .skeletons
                        .iter_mut()
                        .find(|s| s.transform.position == other_teleporter.transform.position);
                    if let Some(skeleton) = skeleton_option {
                        skeleton.ai.state = AiState::Attack;
                    }
                }
            }
            // Exit
            let is_on_exit = game_state.exit.transform.position == player.transform.position;
            let all_skeletons_freed = game_state
                .skeleton_blocks
                .iter()
                .all(|s| s.buried.is_released);
            if is_on_exit {
                if all_skeletons_freed {
                    should_exit = true;
                    sound_collection.play(4);
                } else {
                    sound_collection.play(6);
                    for skeleton_block in game_state.skeleton_blocks.iter_mut() {
                        skeleton_block.sprite.blink_timer = constantes::TIME_BLINK;
                    }
                }
            }
        }
        PlayerInputIntent::Down => {
            let pos_below = player.transform.position + na::Vector2::new(0, 1);
            let skeleton_block_option = game_state
                .skeleton_blocks
                .iter_mut()
                .find(|s| s.transform.position == pos_below);
            if let Some(skeleton_block) = skeleton_block_option {
                skeleton_block.dig();
            }

            let grass_particle_system = particle_collection.get_mut(*grass_id).unwrap();
            let mut pos_particle = na::Vector2::new(
                player.sprite.visual_position.x / screen_size.x * 16.0,
                player.sprite.visual_position.y / screen_size.x * 16.0,
            );

            grass_particle_system.scale = screen_size.x / 16.0;
            // offset to under player
            pos_particle.x += 16.0 * 0.5;
            pos_particle.y += 16.0;
            grass_particle_system.position = pos_particle;
            grass_particle_system.emit(20);

            sound_collection.play(1);
            player.sprite.texture_index = 8;

            // Foilage fly!
            let foilage_index_option = game_state
                .foilages
                .iter()
                .position(|f| f.pos_i32 == pos_below);
            if let Some(foilage_index) = foilage_index_option {
                let foilage = game_state.foilages.remove(foilage_index);
                let foilage_texture_index = foilage.sprite.texture_index;
                let particle_system_index = match foilage_texture_index {
                    14 => *foilage_1_id,
                    15 => *foilage_2_id,
                    16 => *foilage_3_id,
                    17 => *foilage_4_id,
                    _ => *foilage_1_id,
                };
                let foilage_particle_system =
                    particle_collection.get_mut(particle_system_index).unwrap();
                foilage_particle_system.scale = screen_size.x / 16.0;
                foilage_particle_system.position = pos_particle;
                foilage_particle_system.emit(1);
            }
        }
        PlayerInputIntent::None => {}
    }

    let player = &mut game_state.player;
    // bounds check
    if player.transform.position.x < 0 {
        player.transform.position.x = 0;
    } else if player.transform.position.x > constantes::GAME_BOUNDS_X {
        player.transform.position.x = constantes::GAME_BOUNDS_X;
    }

    player.input_intent = PlayerInputIntent::None;
    if should_exit {
        map::clear_map(game_state);
        *current_map += 1;
        if *current_map >= map::MAP_COUNT {
            game_state.is_all_levels_completed = true;
        } else {
            map::load_map(ctx, game_state, *current_map, screen_size);
        }
    }
}
