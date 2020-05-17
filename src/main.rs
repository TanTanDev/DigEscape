mod particle_system;
use particle_system::{ParticleSystem, ValueGetter, VelocityType, AngleData, ParticleSystemCollection};

use std::io::{Read};
use std::f32;
use std::path;
use std::env;

// Magic!
use gwg as ggez;

use mint;
use ggez::{Context, GameResult};
use ggez::conf::*;
use ggez::event;
use ggez::rand;
use ggez::graphics;
use ggez::graphics::{Color, DrawParam, FilterMode};
use nalgebra as na;
use ggez::event::{KeyCode, KeyMods};
use std::collections::HashMap;
use ggez::audio;

const CLEAR_COLOR: Color = Color::new(0.0,0.0,0.0,1.0);
const BACKGROUND_GAME: Color = Color::new(56.0/255.0, 82.0/255.0, 119.0/255.0, 1.0);
const COLOR_BLINK: Color = Color::new(2.0,2.0,2.0,1.0);
const COLOR_BLOOD: Color = Color::new(171.0/255.0, 34.0/255.0, 44.0/255.0, 1.0);
const GAME_SCALE: f32 = 5.0;
const TIME_BLINK: f32 = 0.4;
const TIME_AUTO_STEP: f32 = 0.2;
const TIME_VISUAL_LERP: f32 = 1.0/0.2*2.0;
const GAME_BOUNDS_Y: i32 = 7;
const GAME_BOUNDS_X: i32 = 9;
const GAME_BOUNDS_PADDING: f32 = 5.0; // Warp clouds
const SIZE_FOILAGE_DELTA: f32 = 0.2;
const FOILAGE_SPAWN_CHANCE: f32 = 0.6;
const FOILAGE_BUSH_CHANCE: f32 = 1.0/4.0; // 25% chance to spawn bush, otherwise straw
const ROTATION_FOILAGE_MAX: f32 = 1.0;
const TIME_FOILAGE_SPEED: f32 = 3.0;
const MAX_CLOUDS: i32 = 8;
const MIN_CLOUDS: i32 = 2;
const CLOUD_MIN_SPEED: f32 = 0.1;
const CLOUD_MAX_SPEED: f32 = 0.6;
const CLOUD_MAX_SCALE: f32 = 2.0;
const PI: f32 = std::f32::consts::PI;

enum FoilageType {
    Straw, // Rotates
    Bush, // Stretches
}

struct Foilage {
    position: na::Point2::<f32>,
    sprite: SpriteComponent,
    foilage_type: FoilageType,
    time_offset: f32,
}

impl Foilage {
    fn new(position: na::Point2<f32>) -> Self {
        let is_bush = rand::gen_range(0.0, 1.0) < FOILAGE_BUSH_CHANCE;
        let foilage_type = if is_bush { FoilageType::Bush } else { FoilageType::Straw };
        let texture_index = match foilage_type {
            FoilageType::Straw => rand::gen_range(14,16+1),
            FoilageType::Bush => 17,
        };

        Foilage {
            position,
            sprite: SpriteComponent {
                texture_index,
                scale: na::Vector2::new(1.0, 1.0),
                is_flipped: rand::gen_range(0,2) == 0,
                .. Default::default()
            },
            foilage_type, 
            time_offset: rand::gen_range(0.0, 1.0),
        }
    }
}

struct Cloud {
    sprite: SpriteComponent,
    position: na::Point2::<f32>,
    speed: f32,
}

impl Cloud {
    fn new() -> Self {
        let speed = rand::gen_range(CLOUD_MIN_SPEED, CLOUD_MAX_SPEED);
        let scaleX = rand::gen_range(1.0, CLOUD_MAX_SCALE);
        let scaleY = rand::gen_range(1.0, CLOUD_MAX_SCALE);
        let scale = na::Vector2::new(scaleX, scaleY);
        let texture_index = rand::gen_range(18,20+1);
        let positionX = rand::gen_range(-GAME_BOUNDS_PADDING, GAME_BOUNDS_X as f32 + GAME_BOUNDS_PADDING);
        let positionY = rand::gen_range(0.0, GAME_BOUNDS_Y as f32);
        let position = na::Point2::new(positionX, positionY);
        let sprite = SpriteComponent{ texture_index, scale, ..Default::default()}; 
        Cloud{sprite, position, speed}
    }
}

#[derive(PartialEq)]
enum PlayerInputIntent {
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
struct Player {
    transform: TransformComponent,
    sprite: SpriteComponent,
    input_intent: PlayerInputIntent,
    time_since_step: f32,
    is_alive: bool,
    is_on_skeleton: bool, // Used for other to look at
    prev_grounded: bool,
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
            transform: TransformComponent::default()
        }
    }
}

impl Player {
    fn should_step(&mut self, dt: f32, grasses: &Vec<Grass>
        , skeletons: &Vec<Skeleton>, skeleton_blocks: &Vec<SkeletonBlock>
        , sound_collection: &mut SoundCollection, particle_collection: &mut ParticleSystemCollection
        , land_id: &u32, screen_size: &na::Point2<f32>
        ) -> bool
    {
        if !self.is_alive {
            return false;
        }

        self.time_since_step += dt;
        let pos_below = self.transform.position + na::Vector2::new(0, 1);
        let mut is_grounded = skeletons.iter().any(|s|s.transform.position == pos_below);
        self.is_on_skeleton = is_grounded;
        is_grounded |= grasses.iter().any(|g|g.transform.position == pos_below);
        is_grounded |= skeleton_blocks.iter().any(|s|s.transform.position == pos_below);

        if self.input_intent != PlayerInputIntent::None && is_grounded {
            self.time_since_step = 0.0;
            return true;
        }

        if self.time_since_step > TIME_AUTO_STEP {
            if !is_grounded {
                self.time_since_step = 0.0;
                return true;
            }

            if is_grounded && !self.prev_grounded {
                self.sprite.texture_index = 0;
                self.prev_grounded = true;
                sound_collection.play(8);

                let mut land_particles = particle_collection.get_mut(*land_id).unwrap();
                land_particles.scale = screen_size.x/16.0;
                let pos_player_visual = self.sprite.visual_position;
                let mut pos_particle = na::Vector2::new(
                    pos_player_visual.x/screen_size.x*16.0
                    , pos_player_visual.y/screen_size.x*16.0);
                pos_particle += na::Vector2::new(16.0*0.5, 16.0);

                land_particles.position = pos_particle;
                land_particles.emit(15);
            }
       }
       false
    }
}

#[derive(Default)]
struct Skeleton {
    transform: TransformComponent,
    sprite: SpriteComponent,
    ai: AiComponent,
}

#[derive(Default)]
struct Grass {
    transform: TransformComponent,
    sprite: SpriteComponent,
}

#[derive(Default)]
struct SkeletonBlock {
    transform: TransformComponent,
    sprite: SpriteComponent,
    buried: BuriedComponent,
}

impl SkeletonBlock {
    fn dig(&mut self) {
       self.buried.is_dug = true; 
       self.sprite.texture_index = 2;
    }

    fn try_release(&mut self) -> bool {
        if self.buried.is_released { return false; }
        if !self.buried.is_dug { return false; }
        self.buried.is_released = true;
        true
    }
}

#[derive(Default)]
struct Teleporter {
    transform: TransformComponent,
    sprite: SpriteComponent,
}

#[derive(Default)]
struct Exit {
    transform: TransformComponent,
    sprite: SpriteComponent,
}

struct TransformComponent {
    position: na::Point2<i32>,
}

impl Default for TransformComponent {
    fn default() -> Self { 
        TransformComponent {
            position: na::Point2::new(0, 0)
        }
    }
}

struct SpriteComponent {
    texture_index: usize,
    scale: na::Vector2<f32>,
    is_flipped: bool,
    visual_position: na::Point2<f32>,
    blink_timer: f32,
}

impl Default for SpriteComponent {
    fn default() -> Self {
        SpriteComponent {
            texture_index: 0,
            scale: na::Vector2::new(1.0, 1.0),
            is_flipped: false,
            visual_position: na::Point2::new(0.0,0.0),
            blink_timer: 0.0,
        }
    }
}

#[derive(PartialEq)]
enum AiState {
    Walk,
    Attack,
}

struct AiComponent {
    state: AiState,
    turn_taken: bool,
}

impl Default for AiComponent {
    fn default() -> Self {
        AiComponent {
            state: AiState::Attack,
            turn_taken: false,
        }
    }
}

#[derive(Default)]
struct BuriedComponent {
    is_dug: bool,
    is_released: bool,
}

struct BlackBorder {
    mesh: graphics::Mesh,
    draw_param: graphics::DrawParam,
}

// todo: move
fn render_border(ctx: &mut Context, border: &Option<BlackBorder>) -> GameResult{
    match border {
        Some(b) => graphics::draw(ctx, &b.mesh, b.draw_param)?,
        None => {},
    }
    Ok(())
}

struct GameState {
    player: Player,
    grasses: Vec<Grass>,
    skeleton_blocks: Vec<SkeletonBlock>,
    skeletons: Vec<Skeleton>,
    foilages: Vec<Foilage>,
    clouds: Vec<Cloud>,
    teleporters: [Option<Teleporter>; 2],
    exit: Exit,
    map_size: na::Point2::<f32>,
    game_over_text: ggez::graphics::Text,
    all_levels_completed_text: ggez::graphics::Text,
    is_all_levels_completed: bool,
}

struct SoundCollection {
    sounds: [audio::Source; 10],
    is_on: bool,
}

impl SoundCollection {
    fn play(&mut self, index: usize) -> GameResult<()> {
        if !self.is_on {
            return Ok(());
        }
        if let Some(source) = self.sounds.get_mut(index) {
            source.play()?;
        }
        Err(ggez::error::GameError::SoundError)
    }
}

struct SpriteCollection {
    images: [graphics::Image; 21],
}

impl SpriteCollection {
    fn get_sprite<'a, 'b: 'a>(&'a self, index: usize) -> Option<&'a graphics::Image> {
        self.images.get(index)
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> GameState {
        let font = graphics::Font::new(ctx, "kenny_fontpackage/Fonts/Kenney Mini.ttf").unwrap();
        let mut game_over_text = graphics::Text::new(("PRESS (R) to restart!", font, 60.0));
        let mut all_levels_completed_text = graphics::Text::new(("You completed ALL LEVELS! Press R to play again", font, 30.0));
 
        GameState {
            game_over_text,
            all_levels_completed_text,
            map_size: na::Point2::new(0.0,0.0),
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

struct MainState {
    game_state: GameState,
    sprite_collection: SpriteCollection,
    sound_collection: SoundCollection,
    current_map: usize,
    screen_size: na::Point2::<f32>,
    black_border_left: Option<BlackBorder>,
    black_border_right: Option<BlackBorder>,
    particle_systems: ParticleSystemCollection,
    // Particle system ids
    grass_id: u32,
    step_id: u32,
    blood_id: u32,
    land_id: u32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut images = [
            graphics::Image::new(ctx, "textures/player.png")?,
            graphics::Image::new(ctx, "textures/ground.png")?,
            graphics::Image::new(ctx, "textures/unburied.png")?,
            graphics::Image::new(ctx, "textures/buried.png")?,
            graphics::Image::new(ctx, "textures/skeleton_neutral.png")?,
            graphics::Image::new(ctx, "textures/blue_door.png")?,
            graphics::Image::new(ctx, "textures/red_door.png")?,
            graphics::Image::new(ctx, "textures/skeleton_attack.png")?,
            graphics::Image::new(ctx, "textures/player_dig.png")?,
            graphics::Image::new(ctx, "textures/player_dead.png")?,
            graphics::Image::new(ctx, "textures/ground_below.png")?,
            graphics::Image::new(ctx, "textures/sound_on.png")?,
            graphics::Image::new(ctx, "textures/sound_off.png")?,
            graphics::Image::new(ctx, "textures/player_fall.png")?,
            graphics::Image::new(ctx, "textures/foilage_1.png")?,
            graphics::Image::new(ctx, "textures/foilage_2.png")?,
            graphics::Image::new(ctx, "textures/foilage_3.png")?,
            graphics::Image::new(ctx, "textures/foilage_4.png")?,
            graphics::Image::new(ctx, "textures/cloud_1.png")?,
            graphics::Image::new(ctx, "textures/cloud_2.png")?,
            graphics::Image::new(ctx, "textures/cloud_3.png")?,
        ];

        for img in &mut images {
            img.set_filter(FilterMode::Nearest);
        }

        let sprite_collection = SpriteCollection{
            images
        };

        let mut sounds = [
            audio::Source::new(ctx, "sounds/player_walk.wav")?,
            audio::Source::new(ctx, "sounds/player_dig.wav")?,
            audio::Source::new(ctx, "sounds/player_hit.wav")?,
            audio::Source::new(ctx, "sounds/player_teleport.wav")?,
            audio::Source::new(ctx, "sounds/level_completed.wav")?,
            audio::Source::new(ctx, "sounds/skeleton_attack.wav")?,
            audio::Source::new(ctx, "sounds/door_locked.wav")?,
            audio::Source::new(ctx, "sounds/player_fall.wav")?,
            audio::Source::new(ctx, "sounds/player_land.wav")?,
            audio::Source::new(ctx, "sounds/level_restarted.wav")?,
        ];
        let sound_collection = SoundCollection {
            is_on: true,
            sounds,
        };
        let mut particle_systems = ParticleSystemCollection::new();
        let mut grass_particle_system = ParticleSystem::new(ctx);
        grass_particle_system.start_color = ValueGetter::Range(
            (82.0/255.0,166.0/255.0,32.0/255.0).into()
            , (66.0/255.0, 54.0/255.0,39.0/255.0).into());

        grass_particle_system.velocity_type = VelocityType::Angle(AngleData::new(PI, Some(0.4)));
        grass_particle_system.start_speed = ValueGetter::Range(2.0,3.0);
        grass_particle_system.start_lifetime = ValueGetter::Range(0.4, 0.5);
        grass_particle_system.start_scale = ValueGetter::Range(2.0, 3.4);
        grass_particle_system.start_angular_velocity = ValueGetter::Range(2.0, 30.4);

        let mut step_particle_system = ParticleSystem::new(ctx);
        step_particle_system.start_lifetime = ValueGetter::Range(0.2, 0.3);
        step_particle_system.start_scale = ValueGetter::Range(1.0, 2.4);
        step_particle_system.start_color = ValueGetter::Single((82.0/255.0,166.0/255.0,32.0/255.0).into());

        let mut blood_particle_system = ParticleSystem::new(ctx);
        blood_particle_system.start_lifetime = ValueGetter::Range(0.3, 0.5);
        blood_particle_system.start_scale = ValueGetter::Range(1.0, 10.4);
        blood_particle_system.start_color = ValueGetter::Single(COLOR_BLOOD);

        let mut land_particle_system = ParticleSystem::new(ctx);
        land_particle_system.start_lifetime = ValueGetter::Range(0.3, 0.5);
        land_particle_system.start_speed = ValueGetter::Range(0.3, 1.5);
        land_particle_system.start_scale = ValueGetter::Range(1.0, 4.4);
        land_particle_system.gravity = -1.0;
        land_particle_system.start_color = ValueGetter::Single(ggez::graphics::WHITE);

        let grass_id = particle_systems.add_system(grass_particle_system);
        let step_id = particle_systems.add_system(step_particle_system);
        let blood_id = particle_systems.add_system(blood_particle_system);
        let land_id = particle_systems.add_system(land_particle_system);

        let mut game_state = GameState::new(ctx);
        let mut main_state = MainState {
            sprite_collection,
            sound_collection,
            game_state,
            current_map: 0,
            screen_size : na::Point2::new(0.0, 0.0),
            black_border_left: None,
            black_border_right: None,
            particle_systems,
            grass_id,
            step_id,
            blood_id,
            land_id,
        };

        use ggez::event::EventHandler;
        let (w,h) = ggez::graphics::size(ctx);
        main_state.resize_event(ctx, w, h);
        audio::maybe_create_soundmixer(ctx);

        load_map(ctx, &mut main_state.game_state, 0, &main_state.screen_size);
        Ok(main_state)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(ctx).as_secs_f32();
        if self.game_state.is_all_levels_completed {
            return Ok(());
        }
        self.particle_systems.update(delta);

        update_clouds(&mut self.game_state, ctx);

        let mut should_step = false;
        {
            let player = &mut self.game_state.player;
            should_step = player.should_step(delta, &self.game_state.grasses
                , &self.game_state.skeletons, &self.game_state.skeleton_blocks
                , &mut self.sound_collection, &mut self.particle_systems
                , &self.land_id, &self.screen_size);
        }

        if should_step {
            player_system(&mut self.game_state, ctx, &mut self.current_map
                , &mut self.sound_collection, &self.screen_size
                , &mut self.particle_systems, &self.grass_id
                , &self.step_id);

            skeleton_system(&mut self.game_state, ctx, &mut self.sound_collection
                , &mut self.particle_systems, &self.blood_id, &self.screen_size);

            skeleton_block_system(&mut self.game_state, &mut self.sound_collection);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, CLEAR_COLOR);
        render_system(&mut self.game_state, &self.sprite_collection
            , ctx, &self.screen_size, &self.sound_collection
            , &self.black_border_left, & self.black_border_right);
        self.particle_systems.draw(ctx);
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods, repeat: bool)
    {
        if repeat {
            return;
        }

        let intent = match keycode {
            KeyCode::Right | KeyCode::D => PlayerInputIntent::Right, 
            KeyCode::Left | KeyCode::A => PlayerInputIntent::Left,
            KeyCode::Down | KeyCode::S => PlayerInputIntent::Down, 
            KeyCode::Up | KeyCode::W => PlayerInputIntent::Up, 
            _ => PlayerInputIntent::None,
        };

        self.game_state.player.input_intent = intent;
        match keycode {
            KeyCode::R => {
                if self.game_state.is_all_levels_completed {
                    self.current_map = 0;
                    self.game_state.is_all_levels_completed = false;
                }
                clear_map(&mut self.game_state);
                load_map(ctx, &mut self.game_state, self.current_map, &self.screen_size);
                self.sound_collection.play(9);
            },
            KeyCode::M => {
                self.sound_collection.is_on = !self.sound_collection.is_on;
            }
            _ => {},
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, w: f32, h: f32) {
        // This scaling code is a mess, send halp
        let mapW = 10.0;
        let mapH = 8.0;
        let sprite_scale = (h / mapH).min(w / mapW);
        self.screen_size.x = sprite_scale;
        self.screen_size.y = sprite_scale;
        let offsetX = (w - mapW*sprite_scale)*0.5;
        let offsetY = (h - mapH*sprite_scale)*0.5;
        ggez::graphics::set_screen_coordinates(ctx, ggez::graphics::Rect::new(-offsetX,-offsetY,w,h));
        force_visual_positions(&mut self.game_state, &self.screen_size);

        let border_width = (w - (mapW*sprite_scale))*0.5;
        let border_height = h;
        let border_y = 0.0;
        let left_pos = -border_width;
        let right_pos = w-border_width*2.0;
        update_borders(ctx, &mut self.black_border_left, &mut self.black_border_right
            ,border_width, border_height, left_pos, right_pos, border_y);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: ggez::event::MouseButton, x: f32, y: f32) {
        let screen_rect = ggez::graphics::screen_coordinates(ctx);
        let volume_rect = ggez::graphics::Rect::new(-screen_rect.x, -screen_rect.y, 64.0, 64.0);
        if volume_rect.contains(na::Point2::new(x, y)) {
            self.sound_collection.is_on = !self.sound_collection.is_on; 
        }
    }
}

fn update_borders(ctx: &mut Context, left: &mut Option<BlackBorder>, right: &mut Option<BlackBorder>
    , w: f32, h: f32, left_x:f32, right_x: f32, border_y: f32)
{
    let left_rect = graphics::Rect::new(left_x, border_y, w, h);
    let right_rect = graphics::Rect::new(right_x, border_y, w, h);
    let left_mesh_result = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill()
        , left_rect, CLEAR_COLOR);

    let right_mesh_result = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill()
        , right_rect, CLEAR_COLOR);

    if let Ok(left_mesh) = left_mesh_result {
        *left = Some(BlackBorder{ mesh: left_mesh, draw_param: DrawParam::default()});
    }
    if let Ok(right_mesh) = right_mesh_result {
        *right = Some(BlackBorder{ mesh: right_mesh, draw_param: DrawParam::default()});
    }
}

fn render_sprite(sprite_collection: &SpriteCollection, ctx: &mut Context, transform_component: &TransformComponent,
    sprite: &mut SpriteComponent, screen_size: &na::Point2::<f32>) -> GameResult
{
    let mut offset = mint::Point2{x:0.0, y:0.0};
    let final_scale = sprite.scale.x * screen_size.x;
    let mut flip_scale: f32 = 1.0;
    if sprite.is_flipped {
       flip_scale = -1.0;
       offset.x = 1.0;
    }
    let delta_time = ggez::timer::delta(ctx).as_secs_f32();
    let target_position = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(transform_component.position) * final_scale; 
    sprite.visual_position.x = lerp(sprite.visual_position.x, target_position.x, delta_time*TIME_VISUAL_LERP);
    sprite.visual_position.y = lerp(sprite.visual_position.y, target_position.y, delta_time*TIME_VISUAL_LERP);

    //let dest = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(transform_component.position) * final_scale; 
    let dest = sprite.visual_position;
    let mut params = DrawParam::default()
        .offset(offset)
        .scale(na::Vector2::<f32>::new(flip_scale * final_scale / 16.0, final_scale / 16.0))
        .dest(dest);

    if sprite.blink_timer > 0.0 {
        sprite.blink_timer -= delta_time;
        let fraction = (sprite.blink_timer / TIME_BLINK).sin()*0.5+0.5; 
        let new_color = graphics::Color::new(COLOR_BLINK.r*fraction, COLOR_BLINK.g*fraction,
            COLOR_BLINK.b*fraction, 1.0);
        params = params.color(new_color);
    }

    let image = sprite_collection.images.get(sprite.texture_index).expect("No image with id...");
    graphics::draw(ctx, image, params)?;
    Ok(()) 
}

fn render_clouds(game_state: &mut GameState, sprite_collection: &SpriteCollection
    , ctx: &mut Context, screen_size: &na::Point2::<f32>) -> GameResult
{
    let mut params = DrawParam::default()
        .offset(mint::Point2{x:0.0, y: 0.0});

    for cloud in game_state.clouds.iter() {
        let scale = (cloud.sprite.scale * screen_size.x) / 16.0;
        params = params.scale(scale);
        params = params.dest(cloud.position * screen_size.x);
        let image = sprite_collection.images.get(cloud.sprite.texture_index).expect("No cloud image...");
        graphics::draw(ctx, image, params)?;
    }
    Ok(()) 
}

fn render_game(game_state: &mut GameState, sprite_collection: &SpriteCollection, ctx: &mut Context
    , screen_size: &na::Point2::<f32>, sound_collection: &SoundCollection)
{
    render_clouds(game_state, sprite_collection, ctx, screen_size);
    render_sprite(sprite_collection, ctx, &game_state.exit.transform, &mut game_state.exit.sprite, screen_size);
    for grass in &mut game_state.grasses{
         render_sprite(sprite_collection, ctx, &grass.transform, &mut grass.sprite, screen_size);
    }
    for skeleton_block in &mut game_state.skeleton_blocks {
         render_sprite(sprite_collection, ctx, &skeleton_block.transform, &mut skeleton_block.sprite, screen_size);
    }
    for teleporter_option in game_state.teleporters.iter_mut().map(|t| t.as_mut()) {
        if let Some(teleporter) = teleporter_option {
             render_sprite(sprite_collection, ctx, &teleporter.transform, &mut teleporter.sprite, screen_size);
         }
    }
    for skeleton in game_state.skeletons.iter_mut() {
         render_sprite(sprite_collection, ctx, &skeleton.transform, &mut skeleton.sprite, screen_size);
    }
    render_sprite(sprite_collection, ctx, &game_state.player.transform, &mut game_state.player.sprite, screen_size);
    render_foilage(game_state, sprite_collection, ctx, screen_size);
}

fn render_foilage(game_state: &mut GameState, sprite_collection: &SpriteCollection
    , ctx: &mut Context, screen_size: &na::Point2::<f32>) -> GameResult
{
    for foilage in game_state.foilages.iter_mut() {
        let mut offset = mint::Point2{x:0.5, y:1.0};
        let mut flip_scale = 1.0;
        if foilage.sprite.is_flipped {
            flip_scale = -1.0;
        }
        let dest = foilage.position * screen_size.x;
        let mut time = ggez::timer::time_since_start(ctx).as_secs_f32();
        time *= TIME_FOILAGE_SPEED;
        time += foilage.time_offset;
        let mut scaleX: f32;
        let mut scaleY: f32;

        match foilage.foilage_type {
            FoilageType::Straw => {
                scaleX = 1.0;
                scaleY = 1.0;
            }
            FoilageType::Bush => {
                scaleX = 1.0+(time.sin()*0.5+0.5) * SIZE_FOILAGE_DELTA;
                scaleY = 1.0+(time.cos()*0.5+0.5) * SIZE_FOILAGE_DELTA;
            }
        };
        scaleX *= screen_size.x;
        scaleY *= screen_size.y;

        let rotation = match foilage.foilage_type {
            FoilageType::Straw => (time.sin() * 0.8) * ROTATION_FOILAGE_MAX,
            FoilageType::Bush => 0.0,
        };

        let mut params = DrawParam::default()
            .offset(offset)
            .scale(na::Vector2::<f32>::new(flip_scale * scaleX / 16.0, scaleY / 16.0))
            .rotation(rotation)
            .dest(dest);
        let image = sprite_collection.images.get(foilage.sprite.texture_index).expect("No image with id...");
        graphics::draw(ctx, image, params)?;
    }
    Ok(())
}

fn render_system(game_state: &mut GameState, sprite_collection: &SpriteCollection, ctx: &mut Context
    , screen_size: &na::Point2::<f32>, sound_collection: &SoundCollection
    , left_border: &Option<BlackBorder>, right_border: &Option<BlackBorder>)
{
   render_background(ctx, screen_size);

   if game_state.is_all_levels_completed {
       render_all_levels_completed(game_state, ctx, screen_size);
   } else {
        render_game(game_state, sprite_collection, ctx, screen_size, sound_collection);
        render_game_over(game_state, ctx, screen_size);
   }
   render_border(ctx, left_border);
   render_border(ctx, right_border);
   render_sound_button(ctx, sprite_collection, sound_collection);
}

fn render_background(ctx: &mut Context, screen_size: &na::Point2::<f32>) {
    let screen_coordinates = ggez::graphics::screen_coordinates(ctx);
    let rect = graphics::Rect::new(-screen_coordinates.x*0.0,0.0, screen_size.x*10.0, screen_size.x*8.0);
    let rectMesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, BACKGROUND_GAME).unwrap();
    graphics::draw(ctx, &rectMesh, DrawParam::default());
}

fn render_sound_button(ctx: &mut Context, sprite_collection: &SpriteCollection, sound_collection: &SoundCollection) {
    let params = DrawParam::default()
        //.scale(na::Vector2::<f32>::new(flip_scale * final_scale / 16.0, final_scale / 16.0))
        .dest(na::Point2::new(0.0,0.0));
    let image_index = match sound_collection.is_on {
        true => 11,
        false => 12,
    };
    let image = sprite_collection.images.get(image_index).expect("No image with id...");
    graphics::draw(ctx, image, params);
}

fn render_game_over(game_state: &mut GameState, ctx: &mut Context, screen_size: &na::Point2::<f32>) -> GameResult {
    if !game_state.player.is_alive {
        let time_since_start = ggez::timer::time_since_start(ctx).as_secs_f32();
        let distance = 20.0;
        let speed = 5.0;
        let offsetY = (time_since_start*speed).sin()*distance;
        let offsetX = (time_since_start*speed).cos()*distance;
        render_text(&game_state.game_over_text, ctx, screen_size, na::Vector2::new(offsetX, offsetY))?;
    }
    Ok(())
}

fn render_all_levels_completed(game_state: &mut GameState, ctx: &mut Context, screen_size: &na::Point2::<f32>) -> GameResult {
   if true {
        render_text(&game_state.all_levels_completed_text, ctx, screen_size, na::Vector2::new(0.0,0.0))?;
   }
    Ok(())
}

fn lerp(from: f32, to: f32, dt: f32) -> f32 {
    return from + dt * (to - from)
}

fn render_text(text: &graphics::Text, ctx: &mut Context, screen_size: &na::Point2::<f32>
        , offset: na::Vector2::<f32>) -> GameResult
{
    let screen_rect = ggez::graphics::screen_coordinates(ctx);
    let sizeX = screen_size.x * 10.0;
    let sizeY = screen_size.x * 8.0;
    let mut pos_centered = na::Point2::new(sizeX*0.5, sizeY*0.5);
    let (textW, textH) = text.dimensions(ctx);
    pos_centered.x -= textW as f32 *0.5;
    pos_centered.y -= textH as f32 *0.5;
    graphics::draw(ctx, text, (pos_centered + offset, graphics::WHITE),)?;
    Ok(())
}

const MAP_NAMES: &[&str] = &[
    "/maps/map_first.txt"
    ,"/maps/map_1skeleton.txt"
    ,"/maps/map_2skeleton.txt"
    ,"/maps/map_gravity.txt"
    ,"/maps/map_teleport.txt"
    ,"/maps/map_simple_backtrack.txt"
    ,"/maps/map_follow.txt"
    ,"/maps/map_middleclash.txt"
    ,"/maps/map_2skeleton_intro.txt"
    ,"/maps/map_2skeleton_backtrack.txt"
    ,"maps/map_fall_trap.txt"
    ,"/maps/map_force_stand.txt"
    ,"/maps/map_skeleton_hole.txt"
    ,"/maps/map_maze1.txt"
    ,"/maps/map_follow_2.txt"
    ,"/maps/map_3skeleton.txt"
    ,"/maps/map_3skeleton_3holes.txt"
    ,"/maps/map_middleclash_2.txt"
    ,"/maps/map_skeleton_platform.txt"
    ,"/maps/map_3skeleton_3holes_harder.txt"];
const MAP_COUNT: usize = MAP_NAMES.len();

fn get_map_name(index: usize) -> &'static str {
   MAP_NAMES[index]
}

fn clear_map(game_state: &mut GameState) {
    game_state.grasses.clear();
    game_state.skeletons.clear();
    game_state.skeleton_blocks.clear();
    game_state.foilages.clear();
    game_state.clouds.clear();
    game_state.teleporters[0] = None;
    game_state.teleporters[1] = None;
}

fn load_map(ctx: &mut Context, game_state: &mut GameState, map_index: usize, screen_size: &na::Point2::<f32>) {
    let map_filename = get_map_name(map_index);
    let mut file = ggez::filesystem::open(ctx, map_filename).expect("no map file");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    for char in buffer.chars() {
        // almost every case have a transform, create here to avoid redundant code
        let transform = TransformComponent{ position: na::Point2::new(x,y)};

        match char {
            '0' => {
                game_state.player = Player{transform, is_alive: true, .. Default::default()};
            },
            '1' => {
                game_state.grasses.push(Grass {transform, sprite: SpriteComponent{ texture_index: 1, ..Default::default()}, .. Default::default()});
            },
            '2' => {
                game_state.skeleton_blocks.push(SkeletonBlock {transform, sprite: SpriteComponent{ texture_index: 3, ..Default::default()}, .. Default::default()});
            },
            '3' => {
                let sprite = SpriteComponent{texture_index: 5, ..Default::default()};
                if game_state.teleporters[0].is_none() {
                    game_state.teleporters[0] = (Some(Teleporter {transform, sprite, .. Default::default()}));
                } else {
                    game_state.teleporters[1] = (Some(Teleporter {transform, sprite, .. Default::default()}));
                }
            },
            '4' => {
                game_state.exit = Exit { transform, sprite: SpriteComponent{texture_index: 6, ..Default::default()}, .. Default::default()};
            },
            '5' => {
                game_state.grasses.push(Grass {transform, sprite: SpriteComponent{ texture_index: 10, ..Default::default()}, .. Default::default()});
            },
            '\n' => {
                // -1 because it will increment right after to 0
                x = -1;
                y +=1;
            },
            _=> {},
        }
        x +=1;
    }
    // visual position starts at 0,0
    force_visual_positions(game_state, screen_size);

    // foilage time!
    // chance to spawn foilage on any grass block
    for grass in game_state.grasses.iter().filter(|g| g.sprite.texture_index == 1) {
        if rand::gen_range(0.0,1.0) > FOILAGE_SPAWN_CHANCE {
            continue;
        }
        let foilage_count = rand::gen_range(1,2+1);
        let mut position = na::Point2::new(grass.transform.position.x as f32
            , grass.transform.position.y as f32);

        for _i in 0..foilage_count {
            // Put in middle of block
            if foilage_count == 0 {
                position.x += 0.5;
            } else {
                position.x += 0.33;
            }
            game_state.foilages.push(Foilage::new(position));
        }
    }
    // Clouds generation
    spawn_clouds(game_state);
}

fn spawn_clouds(game_state: &mut GameState) {
    let amount = rand::gen_range(MIN_CLOUDS, MAX_CLOUDS+1);
    for _i in 0..amount {
        game_state.clouds.push(Cloud::new());
    }
}

fn update_clouds(game_state: &mut GameState, ctx: &mut Context) {
    let delta = ggez::timer::delta(ctx).as_secs_f32();
    for cloud in game_state.clouds.iter_mut() {
        cloud.position.x += delta * cloud.speed;
        if cloud.position.x > GAME_BOUNDS_X as f32 + GAME_BOUNDS_PADDING {
            cloud.position.x = -GAME_BOUNDS_PADDING;
        }
    }
}

fn force_visual_positions(game_state: &mut GameState, screen_size: &na::Point2::<f32>) {
    let mut position: na::Point2::<f32>;
    for grasses in game_state.grasses.iter_mut() {
        position = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(grasses.transform.position); 
        grasses.sprite.visual_position = position*screen_size.x;
    }
    for skeleton_block in game_state.skeleton_blocks.iter_mut() {
        position = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(skeleton_block.transform.position); 
        skeleton_block.sprite.visual_position = position*screen_size.x;
    }
    for teleporter_option in game_state.teleporters.iter_mut() {
        if let Some(teleporter) = teleporter_option {
            position = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(teleporter.transform.position); 
            teleporter.sprite.visual_position = position*screen_size.x;
        }
    }
    {
        position = na::convert::<na::Point2::<i32>, na::Point2::<f32>>(game_state.exit.transform.position); 
        game_state.exit.sprite.visual_position = position*screen_size.x;
    }
}

fn skeleton_block_system(game_state: &mut GameState, sound_collection: &mut SoundCollection) {
    for skeleton_block in game_state.skeleton_blocks.iter_mut() {
        let pos_above = skeleton_block.transform.position - na::Vector2::new(0, 1);
        let mut is_occupied = game_state.player.transform.position == pos_above;
        is_occupied |= game_state.skeletons.iter().any(|s|s.transform.position == pos_above);
        if is_occupied {
            continue;
        }
        if skeleton_block.try_release() {
            let transform = TransformComponent{ position: pos_above };
            let sprite = SpriteComponent{texture_index: 7, ..Default::default()};
            let mut new_skeleton = Skeleton {transform, sprite, ..Default::default()};
            let delta_player_x = game_state.player.transform.position.x - new_skeleton.transform.position.x;
            if delta_player_x > 0 {
                new_skeleton.sprite.is_flipped = false;
            } else {
                new_skeleton.sprite.is_flipped = true;
            }
            game_state.skeletons.push(new_skeleton);
            sound_collection.play(5);
       }
    }
}

fn skeleton_reset_turns(game_state: &mut GameState) {
    for skeleton in game_state.skeletons.iter_mut() {
        skeleton.ai.turn_taken = false;
    }
}

fn skeleton_walk(game_state: &mut GameState, sound_collection: &mut SoundCollection)
{
    let pos_player = game_state.player.transform.position;
    let mut skeleton_new_positions = HashMap::new();
    let mut skeleton_wants_attack: Vec<usize> = vec![];
    let mut skeleton_flip_dirs = HashMap::new();

    for (index, skeleton) in game_state.skeletons.iter()
        .enumerate()
        .filter(|(_i, s)| s.ai.state == AiState::Walk && !s.ai.turn_taken)
    {
        let mut new_position = skeleton.transform.position;
        let pos_below = skeleton.transform.position + na::Vector2::new(0,1);
        let mut is_grounded = pos_player == pos_below;
        is_grounded |= game_state.grasses.iter().any(|g|g.transform.position == pos_below);
        is_grounded |= game_state.skeletons.iter().any(|s|s.transform.position == pos_below);
        is_grounded |= game_state.skeleton_blocks.iter().any(|s|s.transform.position == pos_below);
        
        // walk towards player
        if is_grounded {
            let mut pos_skele = skeleton.transform.position;
            if pos_skele.x < pos_player.x {
                pos_skele.x += 1;
                skeleton_flip_dirs.insert(index, false);
            } else if pos_skele.x > pos_player.x {
                pos_skele.x -= 1;
                skeleton_flip_dirs.insert(index, true);
            }
            let mut is_occupied = pos_skele == pos_player;
            if is_occupied {
                skeleton_wants_attack.push(index);
                sound_collection.play(5);
                continue;
            }
            is_occupied |= game_state.grasses.iter().any(|g|g.transform.position == pos_skele);
            is_occupied |= game_state.skeleton_blocks.iter().any(|s|s.transform.position == pos_skele);
            is_occupied |= game_state.skeletons.iter().any(|s|s.transform.position == pos_skele);
            let up_vector = na::Vector2::new(0,-1);
            let is_other_skeleton_falling = game_state.skeletons.iter().any(|s|s.transform.position == pos_skele + up_vector ); 
            
            let pos_above_skeleton = skeleton.transform.position + up_vector;
            let is_skeleton_above = game_state.skeletons.iter().any(|s|s.transform.position == pos_above_skeleton);

            if !is_occupied && !game_state.player.is_on_skeleton && !is_skeleton_above && !is_other_skeleton_falling {
                new_position = pos_skele;
            }
        } else { // handle gravity
            new_position = pos_below;
        }

        in_bounds(&mut new_position);
        let is_occupied = skeleton_new_positions.iter().any(|(i, p)|*p == new_position);
        if is_occupied {
            //skeleton_new_positions.insert(index, skeleton.transform.position);
        } else {
            skeleton_new_positions.insert(index, new_position);
        }
    }
    for (i, skeleton) in game_state.skeletons.iter_mut().enumerate() {
        match skeleton_new_positions.get(&i) {
            Some(p) => {
                skeleton.transform.position = *p;
            }
            None => {},
        }
    }
    for i in skeleton_wants_attack.iter() {
        match game_state.skeletons.get_mut(*i) {
            Some(skeleton) => {
                skeleton.sprite.texture_index = 7;
                skeleton.ai.state = AiState::Attack
            },
            None => {},
        }
    }
    for (i, flipped) in skeleton_flip_dirs.iter() {
        if let Some(skeleton) = game_state.skeletons.get_mut(*i) {
            skeleton.sprite.is_flipped = *flipped;
        }
    }
}

fn skeleton_attack(game_state: &mut GameState, ctx: &mut Context
    , sound_collection: &mut SoundCollection, particle_collection: &mut ParticleSystemCollection
    , blood_id: &u32, screen_size: &na::Point2<f32>)
{
    let player = &mut game_state.player;
    let pos_player = &player.transform.position;
    for skeleton in game_state.skeletons.iter_mut()
        .filter(|s|s.ai.state == AiState::Attack)
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
                sound_collection.play(2);

                let blood_particles = particle_collection.get_mut(*blood_id).unwrap();
                blood_particles.scale = screen_size.x/16.0;
                let pos_player_visual = player.sprite.visual_position;
                let mut pos_particle = na::Vector2::new(
                    pos_player_visual.x/screen_size.x*16.0
                    , pos_player_visual.y/screen_size.x*16.0);
                pos_particle += na::Vector2::new(16.0*0.5, 16.0*0.5);

                blood_particles.position = pos_particle;
                blood_particles.emit(20);
            },
            false => {
                skeleton.ai.state = AiState::Walk;    
                skeleton.sprite.texture_index = 4;
            },
        }
        skeleton.ai.turn_taken = true;
    }
}

fn skeleton_system(game_state: &mut GameState, ctx: &mut Context, sound_collection: &mut SoundCollection
    , particle_collection: &mut ParticleSystemCollection, blood_id: &u32, screen_size: &na::Point2<f32>)
{
    skeleton_attack(game_state, ctx, sound_collection, particle_collection, blood_id, screen_size);
    skeleton_walk(game_state, sound_collection);
    skeleton_reset_turns(game_state);
}

fn in_bounds(position: &mut na::Point2::<i32>) {
    if position.x < 0 { position.x = 0; }
    else if position.x > GAME_BOUNDS_X {position.x = GAME_BOUNDS_X; }
    if position.y > GAME_BOUNDS_Y {position.y = 0}
}

fn emit_step_particle(particle_collection: &mut ParticleSystemCollection, step_id: &u32
    , amount: i32, is_right_dir: bool, player: &Player, screen_size: &na::Point2<f32>)
{
    let step_particle = particle_collection.get_mut(*step_id).unwrap();
    let mut pos_particle = na::Vector2::new(
        player.sprite.visual_position.x/screen_size.x*16.0
        , player.sprite.visual_position.y/screen_size.x*16.0);

    if is_right_dir {
        step_particle.velocity_type = VelocityType::Angle(AngleData::new(-PI*0.8, Some(0.2)));
    } else {
        step_particle.velocity_type = VelocityType::Angle(AngleData::new(PI*0.8, Some(0.2)));
    }
    step_particle.scale = screen_size.x/16.0;
    // offset to under player
    pos_particle.x += 16.0*0.5;
    pos_particle.y += 16.0;
    step_particle.position = pos_particle;

    step_particle.emit(amount);
}

fn player_system(game_state: &mut GameState, ctx: &mut Context
    , current_map: &mut usize, sound_collection: &mut SoundCollection, screen_size: &na::Point2<f32>
    , particle_collection: &mut ParticleSystemCollection, grass_id: &u32, step_id: &u32)
{
    let mut should_exit = false;
    
    let player = &mut game_state.player;
    let pos_below = player.transform.position + na::Vector2::new(0, 1);
    let mut is_grounded = game_state.grasses.iter().any(|g|g.transform.position == pos_below);
    is_grounded |= game_state.skeletons.iter().any(|s|s.transform.position == pos_below);
    is_grounded |= game_state.skeleton_blocks.iter().any(|s|s.transform.position == pos_below);

    if player.prev_grounded && !is_grounded {
        sound_collection.play(7);
        player.sprite.texture_index = 13;
    }

    player.prev_grounded = is_grounded;
    if !is_grounded {
        player.transform.position = pos_below;

        if player.transform.position.y > GAME_BOUNDS_Y {
            player.transform.position.y = 0;
        }
        return;
    }
    player.sprite.texture_index = 0;

    match player.input_intent {
        PlayerInputIntent::Left => {
            player.sprite.is_flipped = true;
            let new_position = player.transform.position - na::Vector2::new(1,0);
            let occupied_by_grass = game_state.grasses.iter().any(|g| g.transform.position == new_position);
            let occupied_by_skeleton = game_state.skeletons.iter().any(|s| s.transform.position == new_position);
            let occupied_by_skeleton_block = game_state.skeleton_blocks.iter().any(|s| s.transform.position == new_position);
            let is_occupied = occupied_by_grass || occupied_by_skeleton || occupied_by_skeleton_block;
            if !is_occupied {
                player.transform.position = new_position;
                sound_collection.play(0);

                emit_step_particle(particle_collection, step_id, 8, false, player, screen_size);
            }
        },
        PlayerInputIntent::Right => {
            player.sprite.is_flipped = false;
            let new_position = player.transform.position + na::Vector2::new(1,0);
            let occupied_by_grass = game_state.grasses.iter().any(|g| g.transform.position == new_position);
            let occupied_by_skeleton = game_state.skeletons.iter().any(|s| s.transform.position == new_position);
            let occupied_by_skeleton_block = game_state.skeleton_blocks.iter().any(|s| s.transform.position == new_position);
            let is_occupied = occupied_by_grass || occupied_by_skeleton || occupied_by_skeleton_block;
            if !is_occupied {
                player.transform.position = new_position;
                sound_collection.play(0);
                emit_step_particle(particle_collection, step_id, 8, true, player, screen_size);
            }
        },
        PlayerInputIntent::Up => {
            // Teleporter
            let teleporter_tuple_option = game_state.teleporters
                .iter().map(|t| t.as_ref())
                .enumerate()
                .find(|(i, t)| {
                    if let Some(teleporter) = t {
                        return teleporter.transform.position == player.transform.position;
                    } false
                });
            if let Some((index, teleporter)) = teleporter_tuple_option {
                let other_teleporter_index = 1-index;
                let other_teleporter_option = game_state.teleporters.get_mut(other_teleporter_index).unwrap();
                if let Some(other_teleporter) = other_teleporter_option {
                    player.transform.position = other_teleporter.transform.position;
                    sound_collection.play(3);
                    player.sprite.blink_timer = TIME_BLINK;
                    other_teleporter.sprite.blink_timer = TIME_BLINK;
                }
            }
            // Exit
            let is_on_exit = game_state.exit.transform.position == player.transform.position;
            let all_skeletons_freed = game_state.skeleton_blocks.iter().all(|s|s.buried.is_released);
            if is_on_exit {
                if all_skeletons_freed {
                    should_exit = true;
                    sound_collection.play(4);
                } else {
                    sound_collection.play(6);
                    for skeleton_block in game_state.skeleton_blocks.iter_mut() {
                        skeleton_block.sprite.blink_timer = TIME_BLINK;
                    }
                }
            }
        },
        PlayerInputIntent::Down => {
            let pos_below = player.transform.position + na::Vector2::new(0, 1);
            let skeleton_block_option = game_state.skeleton_blocks.iter_mut().find(|s| s.transform.position == pos_below);
            if let Some(skeleton_block) = skeleton_block_option {
                skeleton_block.dig();
            }

            let grass_particle_system = particle_collection.get_mut(*grass_id).unwrap();
            let mut pos_particle = na::Vector2::new(
                player.sprite.visual_position.x/screen_size.x*16.0
                , player.sprite.visual_position.y/screen_size.x*16.0);

            grass_particle_system.scale = screen_size.x/16.0;
            // offset to under player
            pos_particle.x += 16.0*0.5;
            pos_particle.y += 16.0;
            grass_particle_system.position = pos_particle;
            grass_particle_system.emit(20);

            sound_collection.play(1);
            player.sprite.texture_index = 8;
        },
        PlayerInputIntent::None => {}
   }
    
    let player = &mut game_state.player;
    // bounds check
    if player.transform.position.x < 0 {
        player.transform.position.x = 0;
    }
    else if player.transform.position.x > GAME_BOUNDS_X {
        player.transform.position.x = GAME_BOUNDS_X;
    }

    player.input_intent = PlayerInputIntent::None;
    if should_exit {
        clear_map(game_state); 
        *current_map +=1;
        if *current_map >= MAP_COUNT {
            game_state.is_all_levels_completed = true;
        } else {
            load_map(ctx, game_state, *current_map, screen_size);
        }
    }
}

fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf{
            cache: ggez::conf::Cache::Tar(include_bytes!("resources.tar").to_vec()),
            loading: ggez::conf::Loading::Embedded,
            ..Default::default()
        }, // conf
        |mut context| Box::new(MainState::new(&mut context).unwrap()),
    ) // ggez::start
}
