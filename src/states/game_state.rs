use crate::entities::player::Player;
use crate::entities::{
    cloud, foilage, skeleton,
    teleporter::{Exit, Teleporter},
};
use cloud::Cloud;
use foilage::{Foilage, Grass};
use ggez::{graphics, Context};
use gwg as ggez;
use nalgebra as na;
use skeleton::{Skeleton, SkeletonBlock};

pub struct GameState {
    pub player: Player,
    pub grasses: Vec<Grass>,
    pub skeleton_blocks: Vec<SkeletonBlock>,
    pub skeletons: Vec<Skeleton>,
    pub foilages: Vec<Foilage>,
    pub clouds: Vec<Cloud>,
    pub teleporters: [Option<Teleporter>; 2],
    pub exit: Exit,
    pub map_size: na::Point2<f32>,
    pub game_over_text: ggez::graphics::Text,
    pub all_levels_completed_text: ggez::graphics::Text,
    pub is_all_levels_completed: bool,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> GameState {
        let font = graphics::Font::new(ctx, "kenny_fontpackage/Fonts/Kenney Mini.ttf").unwrap();
        let game_over_text = graphics::Text::new(("PRESS (R) to restart!", font, 60.0));
        let all_levels_completed_text = graphics::Text::new((
            "You completed ALL LEVELS! Press R to play again",
            font,
            30.0,
        ));

        GameState {
            game_over_text,
            all_levels_completed_text,
            map_size: na::Point2::new(0.0, 0.0),
            player: Player::default(),
            grasses: vec![],
            skeleton_blocks: vec![],
            skeletons: vec![],
            foilages: vec![],
            clouds: vec![],
            teleporters: [None, None],
            exit: Exit::default(),
            is_all_levels_completed: false,
        }
    }
}
