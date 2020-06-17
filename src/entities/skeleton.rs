use crate::constantes;
use crate::entities::ai::{AiComponent, AiState};
use crate::particle_system::ParticleSystemCollection;
use crate::sound_collection::SoundCollection;
use crate::sprite::SpriteComponent;
use crate::states::game_state::GameState;
use crate::transform_compontent::TransformComponent;
use ggez::Context;
use gwg as ggez;
use nalgebra as na;
use std::collections::HashMap;
#[derive(Default)]
pub struct Skeleton {
    pub transform: TransformComponent,
    pub sprite: SpriteComponent,
    pub ai: AiComponent,
}

#[derive(Default)]
pub struct SkeletonBlock {
    pub transform: TransformComponent,
    pub sprite: SpriteComponent,
    pub buried: BuriedComponent,
}

impl SkeletonBlock {
    pub fn dig(&mut self) {
        self.buried.is_dug = true;
        self.sprite.texture_index = 2;
    }

    pub fn try_release(&mut self) -> bool {
        if self.buried.is_released {
            return false;
        }
        if !self.buried.is_dug {
            return false;
        }
        self.buried.is_released = true;
        true
    }
}

pub fn block_system(game_state: &mut GameState, sound_collection: &mut SoundCollection) {
    for block in game_state.skeleton_blocks.iter_mut() {
        let pos_above = block.transform.position - na::Vector2::new(0, 1);
        let mut is_occupied = game_state.player.transform.position == pos_above;
        is_occupied |= game_state
            .skeletons
            .iter()
            .any(|s| s.transform.position == pos_above);
        if is_occupied {
            continue;
        }
        if block.try_release() {
            let transform = TransformComponent {
                position: pos_above,
            };
            let sprite = SpriteComponent {
                texture_index: 7,
                ..Default::default()
            };
            let mut new_skeleton = Skeleton {
                transform,
                sprite,
                ..Default::default()
            };
            let delta_player_x =
                game_state.player.transform.position.x - new_skeleton.transform.position.x;
            if delta_player_x > 0 {
                new_skeleton.sprite.is_flipped = false;
            } else {
                new_skeleton.sprite.is_flipped = true;
            }
            game_state.skeletons.push(new_skeleton);
            let _ = sound_collection.play(5);
        }
    }
}

pub fn reset_turns(game_state: &mut GameState) {
    for skeleton in game_state.skeletons.iter_mut() {
        skeleton.ai.turn_taken = false;
    }
}

pub fn walk(
    game_state: &mut GameState,
    sound_collection: &mut SoundCollection,
    screen_size: &na::Point2<f32>,
) {
    let pos_player = game_state.player.transform.position;
    let mut new_positions = HashMap::new();
    let mut wants_attack: Vec<usize> = vec![];
    let mut skeleton_warped_y: Vec<usize> = vec![];
    let mut flip_dirs = HashMap::new();

    for (index, skeleton) in game_state
        .skeletons
        .iter()
        .enumerate()
        .filter(|(_i, s)| s.ai.state == AiState::Walk && !s.ai.turn_taken)
    {
        let mut new_position = skeleton.transform.position;
        let pos_below = skeleton.transform.position + na::Vector2::new(0, 1);
        let mut is_grounded = pos_player == pos_below;
        is_grounded |= game_state
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

        // walk towards player
        if is_grounded {
            let mut pos_skele = skeleton.transform.position;
            if pos_skele.x < pos_player.x {
                pos_skele.x += 1;
                flip_dirs.insert(index, false);
            } else if pos_skele.x > pos_player.x {
                pos_skele.x -= 1;
                flip_dirs.insert(index, true);
            }
            let mut is_occupied = pos_skele == pos_player;
            if is_occupied {
                wants_attack.push(index);
                let _ = sound_collection.play(5);
                continue;
            }
            is_occupied |= game_state
                .grasses
                .iter()
                .any(|g| g.transform.position == pos_skele);
            is_occupied |= game_state
                .skeleton_blocks
                .iter()
                .any(|s| s.transform.position == pos_skele);
            is_occupied |= game_state
                .skeletons
                .iter()
                .any(|s| s.transform.position == pos_skele);
            let up_vector = na::Vector2::new(0, -1);
            let is_other_falling = game_state
                .skeletons
                .iter()
                .any(|s| s.transform.position == pos_skele + up_vector);

            let pos_above_skeleton = skeleton.transform.position + up_vector;
            let is_above = game_state
                .skeletons
                .iter()
                .any(|s| s.transform.position == pos_above_skeleton);

            if !is_occupied && !game_state.player.is_on_skeleton && !is_above && !is_other_falling {
                new_position = pos_skele;
            }
        } else {
            // handle gravity
            new_position = pos_below;
        }

        let warped_y = in_bounds(&mut new_position);
        if warped_y {
            skeleton_warped_y.push(index);
        }
        let is_occupied = new_positions.iter().any(|(_i, p)| *p == new_position);
        if is_occupied {
            //new_positions.insert(index, skeleton.transform.position);
        } else {
            new_positions.insert(index, new_position);
        }
    }
    for (i, skeleton) in game_state.skeletons.iter_mut().enumerate() {
        match new_positions.get(&i) {
            Some(p) => {
                skeleton.transform.position = *p;
            }
            None => {}
        }
    }
    for i in wants_attack.iter() {
        match game_state.skeletons.get_mut(*i) {
            Some(skeleton) => {
                skeleton.sprite.texture_index = 7;
                skeleton.ai.state = AiState::Attack
            }
            None => {}
        }
    }
    for (i, flipped) in flip_dirs.iter() {
        if let Some(skeleton) = game_state.skeletons.get_mut(*i) {
            skeleton.sprite.is_flipped = *flipped;
        }
    }
    for i in skeleton_warped_y.iter() {
        match game_state.skeletons.get_mut(*i) {
            Some(skeleton) => {
                skeleton.sprite.blink_timer = constantes::TIME_BLINK;
                let position =
                    na::convert::<na::Point2<i32>, na::Point2<f32>>(skeleton.transform.position);
                skeleton.sprite.visual_position = position * screen_size.x;
            }
            None => {}
        }
    }
}

pub fn attack(
    game_state: &mut GameState,
    _ctx: &mut Context,
    sound_collection: &mut SoundCollection,
    particle_collection: &mut ParticleSystemCollection,
    blood_id: &u32,
    screen_size: &na::Point2<f32>,
) {
    let player = &mut game_state.player;
    let pos_player = &player.transform.position;
    for skeleton in game_state
        .skeletons
        .iter_mut()
        .filter(|s| s.ai.state == AiState::Attack)
    {
        let pos_skele = skeleton.transform.position;
        let mut pos_skele_to_player = pos_skele;
        if pos_skele.x < pos_player.x {
            pos_skele_to_player.x += 1;
        } else if pos_skele.x > pos_player.x {
            pos_skele_to_player.x -= 1;
        }
        let attack_player = pos_skele_to_player == *pos_player;
        match attack_player {
            true => {
                player.is_alive = false;
                player.sprite.texture_index = 9;
                skeleton.ai.state = AiState::Walk;
                skeleton.sprite.texture_index = 4;
                let _ = sound_collection.play(2);

                let blood_particles = particle_collection.get_mut(*blood_id).unwrap();
                blood_particles.scale = screen_size.x / 16.0;
                let pos_player_visual = player.sprite.visual_position;
                let mut pos_particle = na::Vector2::new(
                    pos_player_visual.x / screen_size.x * 16.0,
                    pos_player_visual.y / screen_size.x * 16.0,
                );
                pos_particle += na::Vector2::new(16.0 * 0.5, 16.0 * 0.5);

                blood_particles.position = pos_particle;
                blood_particles.emit(20);
            }
            false => {
                skeleton.ai.state = AiState::Walk;
                skeleton.sprite.texture_index = 4;
            }
        }
        skeleton.ai.turn_taken = true;
    }
}

pub fn system(
    game_state: &mut GameState,
    ctx: &mut Context,
    sound_collection: &mut SoundCollection,
    particle_collection: &mut ParticleSystemCollection,
    blood_id: &u32,
    screen_size: &na::Point2<f32>,
) {
    attack(
        game_state,
        ctx,
        sound_collection,
        particle_collection,
        blood_id,
        screen_size,
    );
    walk(game_state, sound_collection, screen_size);
    reset_turns(game_state);
}

// Returns if the skeleton warped y
pub fn in_bounds(position: &mut na::Point2<i32>) -> bool {
    if position.x < 0 {
        position.x = 0;
    } else if position.x > constantes::GAME_BOUNDS_X {
        position.x = constantes::GAME_BOUNDS_X;
    }
    if position.y > constantes::GAME_BOUNDS_Y {
        position.y = 0;
        return true;
    }
    false
}

#[derive(Default)]
pub struct BuriedComponent {
    pub is_dug: bool,
    pub is_released: bool,
}
