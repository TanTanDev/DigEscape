use std::io::{Read};
use std::f32;
use std::path;
use std::env;
use ggez;
use ggez::{Context, GameResult};
use ggez::conf::*;
use ggez::event;
use ggez::graphics;
use ggez::graphics::{Color, DrawParam, FilterMode};
use ggez::nalgebra as na;
use ggez::event::{KeyCode, KeyMods};
use std::collections::HashMap;

const CLEAR_COLOR: Color = Color::new(0.1,0.2,0.3,1.0);
const GAME_SCALE: f32 = 5.0;
const TIME_AUTO_STEP: f32 = 0.1;
const GAME_BOUNDS_Y: i32 = 7;
const GAME_BOUNDS_X: i32 = 9;

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

#[derive(Default)]
struct Player {
    transform: TransformComponent,
    sprite: SpriteComponent,
    input_intent: PlayerInputIntent,
    time_since_step: f32,
    is_alive: bool,
    is_on_skeleton: bool, // Used for other to look at
}

impl Player {
    fn should_step(&mut self, dt: f32, grasses: &Vec<Grass>,
        skeletons: &Vec<Skeleton>, skeleton_blocks: &Vec<SkeletonBlock>) -> bool
    {
        if !self.is_alive {
            return false;
        }

        self.time_since_step += dt;

        if self.input_intent != PlayerInputIntent::None {
            self.time_since_step = 0.0;
            return true;
        }

        if self.time_since_step > TIME_AUTO_STEP {
            let pos_below = self.transform.position + na::Vector2::new(0, 1);
            let mut is_grounded = skeletons.iter().any(|s|s.transform.position == pos_below);
            self.is_on_skeleton = is_grounded;
            is_grounded |= grasses.iter().any(|g|g.transform.position == pos_below);
            is_grounded |= skeleton_blocks.iter().any(|s|s.transform.position == pos_below);

            if !is_grounded {
            self.time_since_step = 0.0;
                return true;
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
}

impl Default for SpriteComponent {
    fn default() -> Self {
        SpriteComponent {
            texture_index: 0,
            scale: na::Vector2::new(1.0, 1.0),
            is_flipped: false,
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

#[derive(Default)]
struct GameState {
    player: Player,
    grasses: Vec<Grass>,
    skeleton_blocks: Vec<SkeletonBlock>,
    skeletons: Vec<Skeleton>,
    teleporters: [Option<Teleporter>; 2],
    exit: Exit,
    game_over_text: ggez::graphics::Text,
}

struct SpriteCollection {
    images: [graphics::Image; 11],
}

impl SpriteCollection {
    fn get_sprite<'a, 'b: 'a>(&'a self, index: usize) -> Option<&'a graphics::Image> {
        self.images.get(index)
    }
}

impl GameState {
    fn new() -> GameState {
        let mut game_over_text = graphics::Text::new("GAME OVER...\nPress <R> to restart!");
        game_over_text.set_font(graphics::Font::default(), graphics::Scale::uniform(60.0));
        game_over_text.set_bounds(na::Point2::new(700.0, f32::INFINITY), graphics::Align::Center);
 
        GameState {
            game_over_text,
            ..Default::default()
        }
    }
}

struct MainState {
    game_state: GameState,
    sprite_collection: SpriteCollection,
    current_map: usize,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut images = [
            graphics::Image::new(ctx, "/player.png")?,
            graphics::Image::new(ctx, "/ground.png")?,
            graphics::Image::new(ctx, "/unburied.png")?,
            graphics::Image::new(ctx, "/buried.png")?,
            graphics::Image::new(ctx, "/skeleton_neutral.png")?,
            graphics::Image::new(ctx, "/blue_door.png")?,
            graphics::Image::new(ctx, "/red_door.png")?,
            graphics::Image::new(ctx, "/skeleton_attack.png")?,
            graphics::Image::new(ctx, "/player_dig.png")?,
            graphics::Image::new(ctx, "/player_dead.png")?,
            graphics::Image::new(ctx, "/ground_below.png")?,
        ];

        for img in &mut images {
            img.set_filter(FilterMode::Nearest);
        }

        let sprite_collection = SpriteCollection{
            images
        };

        let mut game_state = GameState::new();
        load_map(ctx, &mut game_state, 0);
        let mut main_state = MainState{
            sprite_collection,
            game_state,
            current_map: 0,
        };
        Ok(main_state)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(ctx).as_secs_f32();
        let mut should_step = false;
        {
            let player = &mut self.game_state.player;
            should_step = player.should_step(delta, &self.game_state.grasses,
                &self.game_state.skeletons, &self.game_state.skeleton_blocks);
        }
        if should_step {
            player_system(&mut self.game_state, ctx, &mut self.current_map);
            skeleton_system(&mut self.game_state);
            skeleton_block_system(&mut self.game_state);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, CLEAR_COLOR);
        render_system(&mut self.game_state, &self.sprite_collection, ctx);

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymod: KeyMods, repeat: bool)
    {
        if repeat {
            return;
        }

        let intent = match keycode {
            KeyCode::Right => PlayerInputIntent::Right, 
            KeyCode::Left => PlayerInputIntent::Left, 
            KeyCode::Down => PlayerInputIntent::Down, 
            KeyCode::Up => PlayerInputIntent::Up, 
            _ => PlayerInputIntent::None,
        };
        self.game_state.player.input_intent = intent;
        match keycode {
            KeyCode::R => {
                clear_map(&mut self.game_state);
                load_map(ctx, &mut self.game_state, self.current_map);
            },
            _ => {},
        }
    }
}

fn render_sprite(sprite_collection: &SpriteCollection, ctx: &mut Context, transform_component: &TransformComponent,
    sprite: &SpriteComponent) -> GameResult
{
    let mut offset = ggez::mint::Point2{x:0.0, y:0.0};
    let final_scale = sprite.scale.x * GAME_SCALE;
    let mut flip_scale: f32 = 1.0;
    if sprite.is_flipped {
       flip_scale =-1.0;
       offset.x = 1.0;
    }
    let params = DrawParam::default()
        .offset(offset)
        .scale(na::Vector2::<f32>::new(flip_scale * final_scale, final_scale))
        .dest(na::convert::<na::Point2::<i32>, na::Point2::<f32>>(transform_component.position) * GAME_SCALE * 16.0);

    let image = sprite_collection.images.get(sprite.texture_index).expect("No image with id...");
    graphics::draw(ctx, image, params)?;
    Ok(()) 
}

fn render_system(game_state: &mut GameState, sprite_collection: &SpriteCollection, ctx: &mut Context) {
   render_sprite(sprite_collection, ctx, &game_state.exit.transform, &game_state.exit.sprite);
   for grass in &game_state.grasses{
        render_sprite(sprite_collection, ctx, &grass.transform, &grass.sprite);
   }
   for skeleton_block in &game_state.skeleton_blocks {
        render_sprite(sprite_collection, ctx, &skeleton_block.transform, &skeleton_block.sprite);
   }
   for teleporter_option in game_state.teleporters.iter().map(|t| t.as_ref()) {
       if let Some(teleporter) = teleporter_option {
            render_sprite(sprite_collection, ctx, &teleporter.transform, &teleporter.sprite);
        }
   }
   for skeleton in game_state.skeletons.iter() {
        render_sprite(sprite_collection, ctx, &skeleton.transform, &skeleton.sprite);
   }
   render_sprite(sprite_collection, ctx, &game_state.player.transform, &game_state.player.sprite);
   render_game_over(game_state, ctx);
}

fn render_game_over(game_state: &mut GameState, ctx: &mut Context) -> GameResult {
    if !game_state.player.is_alive {
        let (sizeX, sizeY) = ggez::graphics::size(ctx);
        let mut pos_centered = na::Point2::new(sizeX*0.5, sizeY*0.5);
        let (textW, textH) = game_state.game_over_text.dimensions(ctx);
        pos_centered.x -= textW as f32 *0.5;
        pos_centered.y -= textH as f32 *0.5;
        graphics::draw(ctx, &game_state.game_over_text, (pos_centered, graphics::WHITE),)?;
    }
    Ok(())
}

const MAP_NAMES: &[&str] = &["/map.txt","/map2.txt", "/map3.txt", "/map4.txt", "/map5.txt"];
const MAP_COUNT: usize = MAP_NAMES.len();

fn get_map_name(index: usize) -> &'static str {
   MAP_NAMES[index]
}

fn clear_map(game_state: &mut GameState) {
    game_state.grasses.clear();
    game_state.skeletons.clear();
    game_state.skeleton_blocks.clear();
    game_state.teleporters[0] = None;
    game_state.teleporters[1] = None;
}

fn load_map(ctx: &mut Context, game_state: &mut GameState, map_index: usize) {
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
}

fn skeleton_block_system(game_state: &mut GameState) {
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
       }
    }
}

fn skeleton_reset_turns(game_state: &mut GameState) {
    for skeleton in game_state.skeletons.iter_mut() {
        skeleton.ai.turn_taken = false;
    }
}

fn skeleton_walk(game_state: &mut GameState) {
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
                continue;
            }
            is_occupied |= game_state.grasses.iter().any(|g|g.transform.position == pos_skele);
            is_occupied |= game_state.skeleton_blocks.iter().any(|s|s.transform.position == pos_skele);
            is_occupied |= game_state.skeletons.iter().any(|s|s.transform.position == pos_skele);
            if !is_occupied && !game_state.player.is_on_skeleton {
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

fn skeleton_attack(game_state: &mut GameState) {
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
            },
            false => {
                skeleton.ai.state = AiState::Walk;    
                skeleton.sprite.texture_index = 4;
            },
        }
        skeleton.ai.turn_taken = true;
    }
}

fn skeleton_system(game_state: &mut GameState) {
    skeleton_attack(game_state);
    skeleton_walk(game_state);
    skeleton_reset_turns(game_state);
}

fn in_bounds(position: &mut na::Point2::<i32>) {
    if position.x < 0 { position.x = 0; }
    else if position.x > GAME_BOUNDS_X {position.x = GAME_BOUNDS_X; }
    if position.y > GAME_BOUNDS_Y {position.y = 0}
}

fn player_system(game_state: &mut GameState, ctx: &mut Context, current_map: &mut usize) {

    let mut should_exit = false;
    
    let player = &mut game_state.player;
    let pos_below = player.transform.position + na::Vector2::new(0, 1);
    let mut is_grounded = game_state.grasses.iter().any(|g|g.transform.position == pos_below);
    is_grounded |= game_state.skeletons.iter().any(|s|s.transform.position == pos_below);
    is_grounded |= game_state.skeleton_blocks.iter().any(|s|s.transform.position == pos_below);

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
                let other_teleporter_option = game_state.teleporters.get(other_teleporter_index).unwrap();
                if let Some(other_teleporter) = other_teleporter_option {
                    player.transform.position = other_teleporter.transform.position;
                }
            }
            // Exit
            let is_on_exit = game_state.exit.transform.position == player.transform.position;
            let all_skeletons_freed = game_state.skeleton_blocks.iter().all(|s|s.buried.is_released);
            if is_on_exit && all_skeletons_freed {
                should_exit = true;
            }
        },
        PlayerInputIntent::Down => {
            let pos_below = player.transform.position + na::Vector2::new(0, 1);
            let skeleton_block_option = game_state.skeleton_blocks.iter_mut().find(|s| s.transform.position == pos_below);
            if let Some(skeleton_block) = skeleton_block_option {
                skeleton_block.dig();
            }
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
        if *current_map > MAP_COUNT {
            println!("YOU WIN!");
        } else {
            load_map(ctx, game_state, *current_map);
        }
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("Diggity2", "ggez")
        .window_setup(WindowSetup::default().title("Diggity"))
        .add_resource_path(resource_dir);

    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
