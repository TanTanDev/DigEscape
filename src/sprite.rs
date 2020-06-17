use crate::transform_compontent::TransformComponent;
use gwg as ggez;
use nalgebra as na;

use crate::constantes;
use crate::util;
use ggez::{graphics, Context, GameResult};
use graphics::DrawParam;
pub struct SpriteComponent {
    pub texture_index: usize,
    pub scale: na::Vector2<f32>,
    pub is_flipped: bool,
    pub visual_position: na::Point2<f32>,
    pub blink_timer: f32,
}

impl Default for SpriteComponent {
    fn default() -> Self {
        SpriteComponent {
            texture_index: 0,
            scale: na::Vector2::new(1.0, 1.0),
            is_flipped: false,
            visual_position: na::Point2::new(0.0, 0.0),
            blink_timer: 0.0,
        }
    }
}
pub struct SpriteCollection {
    pub images: [graphics::Image; 21],
}

// impl SpriteCollection {
//     pub fn get_sprite<'a, 'b: 'a>(&'a self, index: usize) -> Option<&'a graphics::Image> {
//         self.images.get(index)
//     }
// }

pub fn render(
    sprite_collection: &SpriteCollection,
    ctx: &mut Context,
    transform_component: &TransformComponent,
    sprite: &mut SpriteComponent,
    screen_size: &na::Point2<f32>,
) -> GameResult {
    let mut offset = mint::Point2 { x: 0.0, y: 0.0 };
    let final_scale = sprite.scale.x * screen_size.x;
    let mut flip_scale: f32 = 1.0;
    if sprite.is_flipped {
        flip_scale = -1.0;
        offset.x = 1.0;
    }
    let delta_time = ggez::timer::delta(ctx).as_secs_f32();
    let target_position =
        na::convert::<na::Point2<i32>, na::Point2<f32>>(transform_component.position) * final_scale;
    sprite.visual_position.x = util::lerp(
        sprite.visual_position.x,
        target_position.x,
        delta_time * constantes::TIME_VISUAL_LERP,
    );
    sprite.visual_position.y = util::lerp(
        sprite.visual_position.y,
        target_position.y,
        delta_time * constantes::TIME_VISUAL_LERP,
    );

    //let dest = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(transform_component.position) * final_scale;
    let dest = sprite.visual_position;
    let mut params = DrawParam::default()
        .offset(offset)
        .scale(na::Vector2::<f32>::new(
            flip_scale * final_scale / 16.0,
            final_scale / 16.0,
        ))
        .dest(dest);

    if sprite.blink_timer > 0.0 {
        sprite.blink_timer -= delta_time;
        let fraction = (sprite.blink_timer / constantes::TIME_BLINK).sin() * 0.5 + 0.5;
        let new_color = graphics::Color::new(
            constantes::COLOR_BLINK.r * fraction,
            constantes::COLOR_BLINK.g * fraction,
            constantes::COLOR_BLINK.b * fraction,
            1.0,
        );
        params = params.color(new_color);
    }

    let image = sprite_collection
        .images
        .get(sprite.texture_index)
        .expect("No image with id...");
    graphics::draw(ctx, image, params)?;
    Ok(())
}
