use ggez::graphics::Color;
use gwg as ggez;

pub const GAME_BOUNDS_Y: i32 = 7;
pub const GAME_BOUNDS_X: i32 = 9;
pub const GAME_BOUNDS_PADDING: f32 = 5.0; // Warp clouds

pub const MAX_CLOUDS: i32 = 8;
pub const MIN_CLOUDS: i32 = 2;
pub const CLOUD_MIN_SPEED: f32 = 0.1;
pub const CLOUD_MAX_SPEED: f32 = 0.6;
pub const CLOUD_MAX_SCALE: f32 = 2.0;

pub const FOILAGE_BUSH_CHANCE: f32 = 1.0 / 4.0; // 25% chance to spawn bush, otherwise straw
pub const FOILAGE_SPAWN_CHANCE: f32 = 0.6;
pub const TIME_FOILAGE_SPEED: f32 = 3.0;
pub const SIZE_FOILAGE_DELTA: f32 = 0.2;
pub const ROTATION_FOILAGE_MAX: f32 = 1.0;

pub const CLEAR_COLOR: Color = Color::new(0.0, 0.0, 0.0, 1.0);
pub const BACKGROUND_GAME: Color = Color::new(56.0 / 255.0, 82.0 / 255.0, 119.0 / 255.0, 1.0);
pub const COLOR_BLINK: Color = Color::new(2.0, 2.0, 2.0, 1.0);
pub const COLOR_BLOOD: Color = Color::new(171.0 / 255.0, 34.0 / 255.0, 44.0 / 255.0, 1.0);

pub const TIME_BLINK: f32 = 0.4;
pub const TIME_AUTO_STEP: f32 = 0.2;
pub const TIME_VISUAL_LERP: f32 = 1.0 / 0.2 * 2.0;

pub const GAME_SCALE: f32 = 5.0;

pub const TOUCH_MIN_DELTA: f32 = 10.0;
pub const TEXT_PADDING_SIZE: f32 = 0.3; // fits all text inside screen with this padding in procentage
pub const PI: f32 = std::f32::consts::PI;
