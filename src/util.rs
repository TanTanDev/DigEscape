use crate::constantes;
use crate::states::game_state::GameState;
use ggez::{graphics, Context, GameResult};
use graphics::DrawParam;
use gwg as ggez;
use nalgebra as na;

pub fn lerp(from: f32, to: f32, dt: f32) -> f32 {
    return from + dt * (to - from);
}

pub fn render_text(
    text: &graphics::Text,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
    offset: na::Vector2<f32>,
) -> GameResult {
    let size_x = screen_size.x * 10.0;
    let size_y = screen_size.x * 8.0;
    let mut pos_centered = na::Point2::new(size_x * 0.5, size_y * 0.5);
    let (text_w, text_h) = text.dimensions(ctx);
    let padding_scale = 1.0 - constantes::TEXT_PADDING_SIZE;
    let scale = (size_x / text_w) * padding_scale;
    pos_centered.x -= text_w as f32 * 0.5 * scale;
    pos_centered.y -= text_h as f32 * 0.5 * scale;

    let draw_param = DrawParam {
        dest: (pos_centered + offset).into(),
        offset: mint::Point2 { x: 0.0, y: 0.0 },
        color: graphics::WHITE,
        scale: mint::Vector2 { x: scale, y: scale },
        ..Default::default()
    };
    graphics::draw(ctx, text, draw_param)?;
    Ok(())
}

pub fn force_visual_positions(game_state: &mut GameState, screen_size: &na::Point2<f32>) {
    let mut position: na::Point2<f32>;
    for grasses in game_state.grasses.iter_mut() {
        position = na::convert::<na::Point2<i32>, na::Point2<f32>>(grasses.transform.position);
        grasses.sprite.visual_position = position * screen_size.x;
    }
    for skeleton_block in game_state.skeleton_blocks.iter_mut() {
        position =
            na::convert::<na::Point2<i32>, na::Point2<f32>>(skeleton_block.transform.position);
        skeleton_block.sprite.visual_position = position * screen_size.x;
    }
    for teleporter_option in game_state.teleporters.iter_mut() {
        if let Some(teleporter) = teleporter_option {
            position =
                na::convert::<na::Point2<i32>, na::Point2<f32>>(teleporter.transform.position);
            teleporter.sprite.visual_position = position * screen_size.x;
        }
    }
    {
        position =
            na::convert::<na::Point2<i32>, na::Point2<f32>>(game_state.exit.transform.position);
        game_state.exit.sprite.visual_position = position * screen_size.x;
    }
}

pub struct BlackBorder {
    mesh: graphics::Mesh,
    draw_param: graphics::DrawParam,
}

pub fn render_border(ctx: &mut Context, border: &Option<BlackBorder>) -> GameResult {
    match border {
        Some(b) => graphics::draw(ctx, &b.mesh, b.draw_param)?,
        None => {}
    }
    Ok(())
}

pub fn update_borders(
    ctx: &mut Context,
    left: &mut Option<BlackBorder>,
    right: &mut Option<BlackBorder>,
    w: f32,
    h: f32,
    left_x: f32,
    right_x: f32,
    border_y: f32,
) {
    let left_rect = graphics::Rect::new(left_x, border_y, w, h);
    let right_rect = graphics::Rect::new(right_x, border_y, w, h);
    let left_mesh_result = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        left_rect,
        constantes::CLEAR_COLOR,
    );

    let right_mesh_result = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        right_rect,
        constantes::CLEAR_COLOR,
    );

    if let Ok(left_mesh) = left_mesh_result {
        *left = Some(BlackBorder {
            mesh: left_mesh,
            draw_param: DrawParam::default(),
        });
    }
    if let Ok(right_mesh) = right_mesh_result {
        *right = Some(BlackBorder {
            mesh: right_mesh,
            draw_param: DrawParam::default(),
        });
    }
}
