use crate::constantes;
use crate::sprite;
use crate::states::game_state;
use game_state::GameState;
use nalgebra as na;
use sprite::{SpriteCollection, SpriteComponent};

use ggez::graphics;
use ggez::{rand, Context, GameResult};
use graphics::DrawParam;
use gwg as ggez;

pub struct Cloud {
    pub sprite: SpriteComponent,
    pub position: na::Point2<f32>,
    pub speed: f32,
}

impl Cloud {
    pub fn new() -> Self {
        let speed = rand::gen_range(constantes::CLOUD_MIN_SPEED, constantes::CLOUD_MAX_SPEED);
        let scale_x = rand::gen_range(1.0, constantes::CLOUD_MAX_SCALE);
        let scale_y = rand::gen_range(1.0, constantes::CLOUD_MAX_SCALE);
        let scale = na::Vector2::new(scale_x, scale_y);
        let texture_index = rand::gen_range(18, 20 + 1);
        let position_x = rand::gen_range(
            -constantes::GAME_BOUNDS_PADDING,
            constantes::GAME_BOUNDS_X as f32 + constantes::GAME_BOUNDS_PADDING,
        );
        let position_y = rand::gen_range(0.0, constantes::GAME_BOUNDS_Y as f32);
        let position = na::Point2::new(position_x, position_y);
        let sprite = SpriteComponent {
            texture_index,
            scale,
            ..Default::default()
        };
        Cloud {
            sprite,
            position,
            speed,
        }
    }
}

pub fn render(
    game_state: &mut GameState,
    sprite_collection: &SpriteCollection,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
) -> GameResult {
    let mut params = DrawParam::default().offset(mint::Point2 { x: 0.0, y: 0.0 });

    for cloud in game_state.clouds.iter() {
        let scale = (cloud.sprite.scale * screen_size.x) / 16.0;
        params = params.scale(scale);
        params = params.dest(cloud.position * screen_size.x);
        let image = sprite_collection
            .images
            .get(cloud.sprite.texture_index)
            .expect("No cloud image...");
        graphics::draw(ctx, image, params)?;
    }
    Ok(())
}

pub fn spawn(game_state: &mut GameState) {
    let amount = rand::gen_range(constantes::MIN_CLOUDS, constantes::MAX_CLOUDS + 1);
    for _i in 0..amount {
        game_state.clouds.push(Cloud::new());
    }
}

pub fn update(game_state: &mut GameState, ctx: &mut Context) {
    let delta = ggez::timer::delta(ctx).as_secs_f32();
    for cloud in game_state.clouds.iter_mut() {
        cloud.position.x += delta * cloud.speed;
        if cloud.position.x > constantes::GAME_BOUNDS_X as f32 + constantes::GAME_BOUNDS_PADDING {
            cloud.position.x = -constantes::GAME_BOUNDS_PADDING;
        }
    }
}
