use crate::constantes;
use crate::entities::skeleton;
use crate::particle_system::{
    AngleData, ParticleSystem, ParticleSystemCollection, ValueGetter, VelocityType,
};
use crate::sound_collection::SoundCollection;
use crate::sprite::{self, SpriteCollection};
use crate::states::game_state::GameState;
use crate::util;
use crate::{
    entities::{cloud, foilage, player},
    map,
};
use event::KeyCode;
use ggez::{audio, event, graphics, Context, GameResult};
use graphics::{DrawParam, FilterMode, draw};
use gwg as ggez;
use gwg::input::keyboard::KeyMods;
use nalgebra as na;
use player::PlayerInputIntent;

pub struct MainState {
    pub game_state: GameState,
    pub sprite_collection: SpriteCollection,
    pub sound_collection: SoundCollection,
    pub current_map: usize,
    pub screen_size: na::Point2<f32>,
    pub black_border_left: Option<util::BlackBorder>,
    pub black_border_right: Option<util::BlackBorder>,
    pub particle_systems: ParticleSystemCollection,
    // Particle system ids
    pub grass_id: u32,
    pub step_id: u32,
    pub blood_id: u32,
    pub land_id: u32,
    pub foilage_1_id: u32,
    pub foilage_2_id: u32,
    pub foilage_3_id: u32,
    pub foilage_4_id: u32,
    pub mouse_pos_down: na::Vector2<f32>,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
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

        let sprite_collection = SpriteCollection { images };

        let sounds = [
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
        let mut grass_particle_system = ParticleSystem::new(ctx, None);
        grass_particle_system.start_color = ValueGetter::Range(
            (82.0 / 255.0, 166.0 / 255.0, 32.0 / 255.0).into(),
            (66.0 / 255.0, 54.0 / 255.0, 39.0 / 255.0).into(),
        );

        grass_particle_system.velocity_type =
            VelocityType::Angle(AngleData::new(constantes::PI, Some(0.4)));
        grass_particle_system.start_speed = ValueGetter::Range(2.0, 3.0);
        grass_particle_system.start_lifetime = ValueGetter::Range(0.4, 0.5);
        grass_particle_system.start_scale = ValueGetter::Range(2.0, 3.4);
        grass_particle_system.start_angular_velocity = ValueGetter::Range(2.0, 30.4);

        let mut step_particle_system = ParticleSystem::new(ctx, None);
        step_particle_system.start_lifetime = ValueGetter::Range(0.2, 0.3);
        step_particle_system.start_scale = ValueGetter::Range(1.0, 2.4);
        step_particle_system.start_color =
            ValueGetter::Single((82.0 / 255.0, 166.0 / 255.0, 32.0 / 255.0).into());

        let mut blood_particle_system = ParticleSystem::new(ctx, None);
        blood_particle_system.start_lifetime = ValueGetter::Range(0.3, 0.5);
        blood_particle_system.start_scale = ValueGetter::Range(1.0, 10.4);
        blood_particle_system.start_color = ValueGetter::Single(constantes::COLOR_BLOOD);

        let mut land_particle_system = ParticleSystem::new(ctx, None);
        land_particle_system.start_lifetime = ValueGetter::Range(0.3, 0.5);
        land_particle_system.start_speed = ValueGetter::Range(0.3, 1.5);
        land_particle_system.start_scale = ValueGetter::Range(1.0, 4.4);
        land_particle_system.gravity = -1.0;
        land_particle_system.start_color = ValueGetter::Single(ggez::graphics::WHITE);

        // Foilage particles
        let foilage_1_image = sprite_collection
            .images
            .get(14)
            .expect("no foilage 1 image");
        let foilage_2_image = sprite_collection
            .images
            .get(15)
            .expect("no foilage 2 image");
        let foilage_3_image = sprite_collection
            .images
            .get(16)
            .expect("no foilage 3 image");
        let foilage_4_image = sprite_collection
            .images
            .get(17)
            .expect("no foilage 4 image");

        let mut foilage_1_particle_system = ParticleSystem::new(ctx, Some(foilage_1_image.clone()));
        foilage_1_particle_system.start_color = ValueGetter::Single(graphics::WHITE);
        foilage_1_particle_system.velocity_type =
            VelocityType::Angle(AngleData::new(constantes::PI, Some(0.1)));
        foilage_1_particle_system.start_speed = ValueGetter::Range(2.0, 7.0);
        foilage_1_particle_system.end_scale = 3.0;
        foilage_1_particle_system.start_lifetime = ValueGetter::Single(6.0);
        foilage_1_particle_system.start_scale = ValueGetter::Single(1.0);

        let mut foilage_2_particle_system = ParticleSystem::new(ctx, Some(foilage_2_image.clone()));
        let mut foilage_3_particle_system = ParticleSystem::new(ctx, Some(foilage_3_image.clone()));
        let mut foilage_4_particle_system = ParticleSystem::new(ctx, Some(foilage_4_image.clone()));
        foilage_2_particle_system.copy_settings(&foilage_1_particle_system);
        foilage_3_particle_system.copy_settings(&foilage_1_particle_system);
        foilage_4_particle_system.copy_settings(&foilage_1_particle_system);

        let grass_id = particle_systems.add_system(grass_particle_system);
        let step_id = particle_systems.add_system(step_particle_system);
        let blood_id = particle_systems.add_system(blood_particle_system);
        let land_id = particle_systems.add_system(land_particle_system);
        let foilage_1_id = particle_systems.add_system(foilage_1_particle_system);
        let foilage_2_id = particle_systems.add_system(foilage_2_particle_system);
        let foilage_3_id = particle_systems.add_system(foilage_3_particle_system);
        let foilage_4_id = particle_systems.add_system(foilage_4_particle_system);

        let game_state = GameState::new(ctx);
        let mut main_state = MainState {
            sprite_collection,
            sound_collection,
            game_state,
            current_map: 0,
            screen_size: na::Point2::new(0.0, 0.0),
            black_border_left: None,
            black_border_right: None,
            particle_systems,
            grass_id,
            step_id,
            blood_id,
            land_id,
            foilage_1_id,
            foilage_2_id,
            foilage_3_id,
            foilage_4_id,
            mouse_pos_down: na::Vector2::new(0.0, 0.0),
        };

        use ggez::event::EventHandler;
        let (w, h) = ggez::graphics::size(ctx);
        main_state.resize_event(ctx, w, h);
        audio::maybe_create_soundmixer(ctx);

        map::load_map(ctx, &mut main_state.game_state, 0, &main_state.screen_size);
        Ok(main_state)
    }
    pub fn restart_current_map(&mut self, ctx: &mut Context) {
        if self.game_state.is_all_levels_completed {
            self.current_map = 0;
            self.game_state.is_all_levels_completed = false;
        }
        map::clear_map(&mut self.game_state);
        map::load_map(
            ctx,
            &mut self.game_state,
            self.current_map,
            &self.screen_size,
        );
        let _ = self.sound_collection.play(9);
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(ctx).as_secs_f32();
        if self.game_state.is_all_levels_completed {
            return Ok(());
        }
        self.particle_systems.update(delta);

        cloud::update(&mut self.game_state, ctx);

        let player = &mut self.game_state.player;
        let should_step = player.should_step(
            delta,
            &self.game_state.grasses,
            &self.game_state.skeletons,
            &self.game_state.skeleton_blocks,
            &mut self.sound_collection,
            &mut self.particle_systems,
            &self.land_id,
            &self.screen_size,
        );

        if should_step {
            player::system(
                &mut self.game_state,
                ctx,
                &mut self.current_map,
                &mut self.sound_collection,
                &self.screen_size,
                &mut self.particle_systems,
                &self.grass_id,
                &self.step_id,
                &self.foilage_1_id,
                &self.foilage_2_id,
                &self.foilage_3_id,
                &self.foilage_4_id,
            );

            skeleton::system(
                &mut self.game_state,
                ctx,
                &mut self.sound_collection,
                &mut self.particle_systems,
                &self.blood_id,
                &self.screen_size,
            );

            skeleton::block_system(&mut self.game_state, &mut self.sound_collection);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, constantes::CLEAR_COLOR);
        render_system(
            &mut self.game_state,
            &self.sprite_collection,
            ctx,
            &self.screen_size,
            &self.sound_collection,
            &self.black_border_left,
            &self.black_border_right,
        );
        self.particle_systems.draw(ctx).unwrap();
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
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
                self.restart_current_map(ctx);
            }
            KeyCode::M => {
                self.sound_collection.is_on = !self.sound_collection.is_on;
            }
            _ => {}
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, w: f32, h: f32) {
        // This scaling code is a mess, send halp
        let map_w = 10.0;
        let map_h = 8.0;
        let sprite_scale = (h / map_h).min(w / map_w);
        self.screen_size.x = sprite_scale;
        self.screen_size.y = sprite_scale;
        let offset_x = (w - map_w * sprite_scale) * 0.5;
        let offset_y = (h - map_h * sprite_scale) * 0.5;
        graphics::set_screen_coordinates(
            ctx,
            ggez::graphics::Rect::new(-offset_x, -offset_y, w, h),
        ).unwrap();
        util::force_visual_positions(&mut self.game_state, &self.screen_size);

        let border_width = (w - (map_w * sprite_scale)) * 0.5;
        let border_height = h;
        let border_y = 0.0;
        let left_pos = -border_width;
        let right_pos = w - border_width * 2.0;
        util::update_borders(
            ctx,
            &mut self.black_border_left,
            &mut self.black_border_right,
            border_width,
            border_height,
            left_pos,
            right_pos,
            border_y,
        );
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.mouse_pos_down = na::Vector2::new(x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        _button: ggez::event::MouseButton,
        x: f32,
        y: f32,
    ) {
        let current_pos = na::Point2::new(x, y);
        // Mute button
        let screen_rect = ggez::graphics::screen_coordinates(ctx);
        let volume_rect = ggez::graphics::Rect::new(-screen_rect.x, -screen_rect.y, 64.0, 64.0);
        if volume_rect.contains(current_pos) {
            self.sound_collection.is_on = !self.sound_collection.is_on;
        }

        let current_pos: na::Vector2<f32> = na::Vector2::new(current_pos.x, current_pos.y);
        let delta = current_pos - self.mouse_pos_down;

        // Restart input Currently tap anywhere on screen if delta is below move action
        if !self.game_state.player.is_alive && delta.norm() < constantes::TOUCH_MIN_DELTA {
            self.restart_current_map(ctx);
        }

        // touch input
        let mut input_intent = PlayerInputIntent::None;
        if delta.norm() > constantes::TOUCH_MIN_DELTA {
            let x_diff = delta.x.abs();
            let y_diff = delta.y.abs();
            if x_diff > y_diff {
                if delta.x > 0.0 {
                    input_intent = PlayerInputIntent::Right;
                } else {
                    input_intent = PlayerInputIntent::Left;
                }
            } else {
                if delta.y > 0.0 {
                    input_intent = PlayerInputIntent::Down;
                } else {
                    input_intent = PlayerInputIntent::Up;
                }
            }
        }
        self.game_state.player.input_intent = input_intent;
    }
}
fn render_game(
    game_state: &mut GameState,
    sprite_collection: &SpriteCollection,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
    _sound_collection: &SoundCollection,
) {
    cloud::render(game_state, sprite_collection, ctx, screen_size).unwrap();
    sprite::render(
        sprite_collection,
        ctx,
        &game_state.exit.transform,
        &mut game_state.exit.sprite,
        screen_size,
    ).unwrap();
    for grass in &mut game_state.grasses {
        sprite::render(
            sprite_collection,
            ctx,
            &grass.transform,
            &mut grass.sprite,
            screen_size,
        ).unwrap();
    }
    for skeleton_block in &mut game_state.skeleton_blocks {
        sprite::render(
            sprite_collection,
            ctx,
            &skeleton_block.transform,
            &mut skeleton_block.sprite,
            screen_size,
        ).unwrap();
    }
    for teleporter_option in game_state.teleporters.iter_mut().map(|t| t.as_mut()) {
        if let Some(teleporter) = teleporter_option {
            sprite::render(
                sprite_collection,
                ctx,
                &teleporter.transform,
                &mut teleporter.sprite,
                screen_size,
            ).unwrap();
        }
    }
    for skeleton in game_state.skeletons.iter_mut() {
        sprite::render(
            sprite_collection,
            ctx,
            &skeleton.transform,
            &mut skeleton.sprite,
            screen_size,
        ).unwrap();
    }
    sprite::render(
        sprite_collection,
        ctx,
        &game_state.player.transform,
        &mut game_state.player.sprite,
        screen_size,
    ).unwrap();
    foilage::render(game_state, sprite_collection, ctx, screen_size).unwrap();
}

fn render_system(
    game_state: &mut GameState,
    sprite_collection: &SpriteCollection,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
    sound_collection: &SoundCollection,
    left_border: &Option<util::BlackBorder>,
    right_border: &Option<util::BlackBorder>,
) {
    render_background(ctx, screen_size);

    if game_state.is_all_levels_completed {
        render_all_levels_completed(game_state, ctx, screen_size).unwrap();
    } else {
        render_game(
            game_state,
            sprite_collection,
            ctx,
            screen_size,
            sound_collection,
        );
        render_game_over(game_state, ctx, screen_size).unwrap();
    }
    util::render_border(ctx, left_border).unwrap();
    util::render_border(ctx, right_border).unwrap();
    render_sound_button(ctx, sprite_collection, sound_collection);
}

fn render_background(ctx: &mut Context, screen_size: &na::Point2<f32>) {
    let screen_coordinates = ggez::graphics::screen_coordinates(ctx);
    let rect = graphics::Rect::new(
        -screen_coordinates.x * 0.0,
        0.0,
        screen_size.x * 10.0,
        screen_size.x * 8.0,
    );
    let rect_mesh = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        rect,
        constantes::BACKGROUND_GAME,
    )
    .unwrap();
    draw(ctx, &rect_mesh, DrawParam::default()).unwrap();
}

fn render_sound_button(
    ctx: &mut Context,
    sprite_collection: &SpriteCollection,
    sound_collection: &SoundCollection,
) {
    let params = DrawParam::default()
        //.scale(na::Vector2::<f32>::new(flip_scale * final_scale / 16.0, final_scale / 16.0))
        .dest(na::Point2::new(0.0, 0.0));
    let image_index = match sound_collection.is_on {
        true => 11,
        false => 12,
    };
    let image = sprite_collection
        .images
        .get(image_index)
        .expect("No image with id...");
    draw(ctx, image, params).unwrap();
}

fn render_game_over(
    game_state: &mut GameState,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
) -> GameResult {
    if !game_state.player.is_alive {
        let time_since_start = ggez::timer::time_since_start(ctx).as_secs_f32();
        let distance = 20.0;
        let speed = 5.0;
        let offset_y = (time_since_start * speed).sin() * distance;
        let offset_x = (time_since_start * speed).cos() * distance;
        util::render_text(
            &game_state.game_over_text,
            ctx,
            screen_size,
            na::Vector2::new(offset_x, offset_y),
        )?;
    }
    Ok(())
}

fn render_all_levels_completed(
    game_state: &mut GameState,
    ctx: &mut Context,
    screen_size: &na::Point2<f32>,
) -> GameResult {
    if true {
        util::render_text(
            &game_state.all_levels_completed_text,
            ctx,
            screen_size,
            na::Vector2::new(0.0, 0.0),
        )?;
    }
    Ok(())
}
