use crate::constantes;
use crate::sprite;
use crate::states::game_state::GameState;
use crate::transform_compontent::TransformComponent;
use nalgebra as na;
use sprite::{SpriteCollection, SpriteComponent};

use ggez::graphics;
use ggez::{rand, Context, GameResult};
use graphics::DrawParam;
use gwg as ggez;
#[derive(Default)]
pub struct Grass {
    pub transform: TransformComponent,
    pub sprite: SpriteComponent,
}
pub enum FoilageType {
    Straw, // Rotates
    Bush,  // Stretches
}

pub struct Foilage {
    pub position: na::Point2<f32>,
    pub pos_i32: na::Point2<i32>, // belongs to this grasss, position
    pub sprite: SpriteComponent,
    pub foilage_type: FoilageType,
    pub time_offset: f32,
}

impl Foilage {
    pub fn new(position: na::Point2<f32>, pos_i32: na::Point2<i32>) -> Self {
        let is_bush = rand::gen_range(0.0, 1.0) < constantes::FOILAGE_BUSH_CHANCE;
        let foilage_type = if is_bush {
            FoilageType::Bush
        } else {
            FoilageType::Straw
        };
        let texture_index = match foilage_type {
            FoilageType::Straw => rand::gen_range(14, 16 + 1),
            FoilageType::Bush => 17,
        };

        Foilage {
            position,
            pos_i32,
            sprite: SpriteComponent {
                texture_index,
                scale: na::Vector2::new(1.0, 1.0),
                is_flipped: rand::gen_range(0, 2) == 0,
                ..Default::default()
            },
            foilage_type,
            time_offset: rand::gen_range(0.0, 1.0),
        }
    }
}
pub fn generate(game_state: &mut GameState) {
    // foilage time!
    // chance to spawn foilage on any grass block
    for grass in game_state
        .grasses
        .iter()
        .filter(|g| g.sprite.texture_index == 1)
    {
        if rand::gen_range(0.0, 1.0) > constantes::FOILAGE_SPAWN_CHANCE {
            continue;
        }
        let foilage_count = rand::gen_range(1, 2 + 1);
        let mut position = na::Point2::new(
            grass.transform.position.x as f32,
            grass.transform.position.y as f32,
        );

        for _i in 0..foilage_count {
            // Put in middle of block
            if foilage_count == 0 {
                position.x += 0.5;
            } else {
                position.x += 0.33;
            }
            game_state
                .foilages
                .push(Foilage::new(position, grass.transform.position));
        }
    }
}
pub fn render(
    game_state: &mut GameState,
    sprite_collection: &SpriteCollection,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
) -> GameResult {
    for foilage in game_state.foilages.iter_mut() {
        let offset = mint::Point2 { x: 0.5, y: 1.0 };
        let mut flip_scale = 1.0;
        if foilage.sprite.is_flipped {
            flip_scale = -1.0;
        }
        let dest = foilage.position * screen_size.x;
        let mut time = ggez::timer::time_since_start(ctx).as_secs_f32();
        time *= constantes::TIME_FOILAGE_SPEED;
        time += foilage.time_offset;
        let mut scale_x: f32;
        let mut scale_y: f32;

        match foilage.foilage_type {
            FoilageType::Straw => {
                scale_x = 1.0;
                scale_y = 1.0;
            }
            FoilageType::Bush => {
                scale_x = 1.0 + (time.sin() * 0.5 + 0.5) * constantes::SIZE_FOILAGE_DELTA;
                scale_y = 1.0 + (time.cos() * 0.5 + 0.5) * constantes::SIZE_FOILAGE_DELTA;
            }
        };
        scale_x *= screen_size.x;
        scale_y *= screen_size.y;

        let rotation = match foilage.foilage_type {
            FoilageType::Straw => (time.sin() * 0.8) * constantes::ROTATION_FOILAGE_MAX,
            FoilageType::Bush => 0.0,
        };

        let params = DrawParam::default()
            .offset(offset)
            .scale(na::Vector2::<f32>::new(
                flip_scale * scale_x / 16.0,
                scale_y / 16.0,
            ))
            .rotation(rotation)
            .dest(dest);
        let image = sprite_collection
            .images
            .get(foilage.sprite.texture_index)
            .expect("No image with id...");
        graphics::draw(ctx, image, params)?;
    }
    Ok(())
}
