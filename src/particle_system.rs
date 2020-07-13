use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::{DrawParam, Image};
use ggez::Context;
use ggez::GameResult;
use gwg as ggez;
use gwg::rand;
use nalgebra as na;
use std::collections::HashMap;
use std::collections::VecDeque;

static DEFAULT_CAPACITY: usize = 8;
static PI: f32 = std::f32::consts::PI;
static TAU: f32 = PI * 2.0;

use na::Point2;
use na::Vector2;

// Todo, since start_lifetime can be randomized, the scaling will not start at start_scale
// because the lifetime fraction also then is randomized

// helper funcitons
// in radians
fn vec_from_angle(angle: f32) -> na::Vector2<f32> {
    na::Vector2::new(angle.sin(), angle.cos())
}

fn lerp(from: f32, to: f32, delta: f32) -> f32 {
    (1.0 - delta) * from + delta * to
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum TransformSpace {
    Local,
    World,
}

pub struct ParticleSystem {
    // Particle data
    positions: Vec<Point2<f32>>,
    velocities: Vec<Vector2<f32>>,
    angular_velocities: Vec<f32>,
    scales: Vec<f32>,
    rotations: Vec<f32>,
    lifetimes: Vec<f32>,
    colors: Vec<ggez::graphics::Color>,
    particle_indexes: VecDeque<usize>,
    available_indexes: VecDeque<usize>,

    // System data
    pub emit_shape: EmitShape,
    pub velocity_type: VelocityType,
    pub gravity: f32,
    pub transform_space: TransformSpace,

    pub scale: f32,
    pub position: Vector2<f32>,
    pub start_lifetime: ValueGetter<f32>,
    pub start_speed: ValueGetter<f32>,
    pub start_rotation: ValueGetter<f32>,
    pub start_scale: ValueGetter<f32>,
    pub start_angular_velocity: ValueGetter<f32>,
    pub start_color: ValueGetter<ggez::graphics::Color>,
    pub end_scale: f32,

    sprite_batch_dirty: bool,
    pub sprite_batch: SpriteBatch,
}

fn make_image(ctx: &mut Context) -> Image {
    // 1 pixel texture with 1.0 in every color
    let bytes = [u8::MAX; 4];
    Image::from_rgba8(ctx, 1, 1, &bytes).unwrap()
}

impl ParticleSystem {
    pub fn copy_settings(&mut self, other: &ParticleSystem) {
        self.emit_shape = other.emit_shape;
        self.velocity_type = other.velocity_type;
        self.gravity = other.gravity;
        self.transform_space = other.transform_space;
        self.scale = other.scale;
        self.position = other.position;
        self.start_lifetime = other.start_lifetime;
        self.start_speed = other.start_speed;
        self.start_rotation = other.start_rotation;
        self.start_scale = other.start_scale;
        self.start_angular_velocity = other.start_angular_velocity;
        self.start_color = other.start_color;
        self.end_scale = other.end_scale;
    }

    pub fn new(ctx: &mut Context, image_option: Option<Image>) -> Self {
        let mut available_indexes = VecDeque::with_capacity(DEFAULT_CAPACITY);
        for i in 0..available_indexes.capacity() {
            available_indexes.push_back(i);
        }

        let final_image;
        if let Some(image) = image_option {
            final_image = image;
        } else {
            final_image = make_image(ctx);
        }

        let sprite_batch = SpriteBatch::new(final_image);

        let mut particle_system = ParticleSystem {
            positions: Vec::with_capacity(DEFAULT_CAPACITY),
            velocities: Vec::with_capacity(DEFAULT_CAPACITY),
            scales: Vec::with_capacity(DEFAULT_CAPACITY),
            lifetimes: Vec::with_capacity(DEFAULT_CAPACITY),
            rotations: Vec::with_capacity(DEFAULT_CAPACITY),
            colors: Vec::with_capacity(DEFAULT_CAPACITY),
            angular_velocities: Vec::with_capacity(DEFAULT_CAPACITY),
            particle_indexes: VecDeque::with_capacity(DEFAULT_CAPACITY),
            available_indexes,

            emit_shape: EmitShape::Point,
            velocity_type: VelocityType::Angle(AngleData::new(PI, Some(0.5))),
            gravity: -9.0,
            transform_space: TransformSpace::World,

            scale: 1.0,
            position: Vector2::new(200.0, 300.0),
            start_lifetime: ValueGetter::Single(1.4),
            start_speed: ValueGetter::Range(0.0, 3.0),
            start_rotation: ValueGetter::Single(0.0),
            start_scale: ValueGetter::Single(16.0),
            start_angular_velocity: ValueGetter::Range(-1.0, 1.0),
            start_color: ValueGetter::Range(
                ggez::graphics::Color::new(0.5, 0.2, 0.2, 1.0),
                ggez::graphics::Color::new(1.0, 1.0, 0.2, 1.0),
            ),

            end_scale: 0.0,

            sprite_batch_dirty: true,
            sprite_batch,
        };
        let available_indexes = particle_system.available_indexes.len();
        for _i in 0..available_indexes {
            particle_system.positions.push(Point2::new(0.0, 0.0));
        }
        for _i in 0..available_indexes {
            particle_system.velocities.push(Vector2::new(0.0, 0.0));
        }
        for _i in 0..available_indexes {
            particle_system.scales.push(1.0);
        }
        for _i in 0..available_indexes {
            particle_system.lifetimes.push(0.0);
        }
        for _i in 0..available_indexes {
            particle_system.rotations.push(0.0);
        }
        for _i in 0..available_indexes {
            particle_system.angular_velocities.push(0.0);
        }
        for _i in 0..available_indexes {
            particle_system.colors.push(ggez::graphics::WHITE);
        }
        particle_system
    }

    // Draw delegate, recieves
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.sprite_batch_dirty {
            return Ok(());
        }
        if self.particle_indexes.len() == 0 {
            return Ok(());
        }
        self.sprite_batch.clear();
        for i in self.particle_indexes.iter() {
            let scale = self.scales[*i];
            let mut dest = self.positions[*i];
            if let TransformSpace::Local = self.transform_space {
                dest += self.position;
            }

            let drawparam = DrawParam {
                offset: Point2::new(0.5, 0.5).into(),
                dest: (dest * self.scale).into(),
                scale: mint::Vector2 {
                    x: scale * self.scale,
                    y: scale * self.scale,
                },
                rotation: self.rotations[*i],
                color: self.colors[*i],
                ..Default::default()
            };
            self.sprite_batch.add(drawparam);
        }
        ggez::graphics::draw(ctx, &self.sprite_batch, DrawParam::default())?;

        self.sprite_batch_dirty = true;
        Ok(())
    }

    pub fn update(&mut self, dt: f32) {
        // Borrowing here to make iterator borrowing easier
        let particle_indexes = &mut self.particle_indexes;
        let lifetimes = &mut self.lifetimes;
        let available_indexes = &mut self.available_indexes;

        // remove particles with lifetime less than 0
        // to those, removed, add that index to available_indexes
        particle_indexes.retain(|i| {
            lifetimes[*i] -= dt;
            if lifetimes[*i] < 0.0 {
                available_indexes.push_back(*i);
                return false;
            }
            true
        });

        for i in self.particle_indexes.iter() {
            self.velocities[*i].y -= self.gravity * dt;
            self.positions[*i] += self.velocities[*i];
            self.rotations[*i] += self.angular_velocities[*i] * dt;
            let normalized_life =
                (self.start_lifetime.max() - lifetimes[*i]) / self.start_lifetime.max();
            self.scales[*i] = lerp(self.start_scale.max(), self.end_scale, normalized_life);
        }
        self.sprite_batch_dirty = true;
    }

    pub fn emit(&mut self, amount: i32) {
        let mut amount = amount;
        //for i in 0..amount {
        while amount > 0 {
            let index_option = self.available_indexes.pop_front();
            match index_option {
                Some(index) => {
                    // make unused particle come alive
                    self.particle_setup(index);
                }
                None => {
                    // Resize vectors and spawn a new particle
                    let left_to_create = amount;
                    self.grow(left_to_create as usize);
                    // we still have a particle to spawn
                    amount += 1;
                }
            }
            amount -= 1;
        }
    }

    // Returns the first available index
    fn grow(&mut self, additional: usize) {
        self.lifetimes.reserve(additional);
        self.positions.reserve(additional);
        self.velocities.reserve(additional);
        self.rotations.reserve(additional);
        self.scales.reserve(additional);
        self.angular_velocities.reserve(additional);
        self.particle_indexes.reserve(additional);
        self.available_indexes.reserve(additional);
        self.colors.reserve(additional);

        let next_available_index = self.lifetimes.len();

        for _i in self.positions.len()..self.positions.capacity() {
            self.positions.push(Point2::new(0.0, 0.0));
        }
        for _i in self.velocities.len()..self.velocities.capacity() {
            self.velocities.push(Vector2::new(0.0, 0.0));
        }
        for _i in self.scales.len()..self.scales.capacity() {
            self.scales.push(0.0);
        }
        for _i in self.lifetimes.len()..self.lifetimes.capacity() {
            self.lifetimes.push(0.0);
        }
        for _i in self.rotations.len()..self.rotations.capacity() {
            self.rotations.push(0.0);
        }
        for _i in self.angular_velocities.len()..self.angular_velocities.capacity() {
            self.angular_velocities.push(0.0);
        }
        for _i in self.colors.len()..self.colors.capacity() {
            self.colors.push(ggez::graphics::WHITE);
        }

        let newly_added = self.lifetimes.len() - next_available_index;
        for i in 0..newly_added {
            self.available_indexes.push_back(next_available_index + i);
        }
    }

    // Setup the data for a newly created particle
    // index is assumed to be in bounds
    fn particle_setup(&mut self, index: usize) {
        let mut pos = self.emit_shape.get_position();
        if let TransformSpace::World = self.transform_space {
            pos += self.position;
        }
        let rotation = self.start_rotation.get();
        let angular_velocity = self.start_angular_velocity.get();
        let scale = self.start_scale.get();
        let speed = self.start_speed.get();
        let direction = self.emit_shape.get_direction(&self.velocity_type, &pos);
        let velocity = direction * speed;
        let lifetime = self.start_lifetime.get();
        let color = self.start_color.get();

        self.lifetimes[index] = lifetime;
        self.positions[index] = pos;
        self.velocities[index] = velocity;
        self.rotations[index] = rotation;
        self.scales[index] = scale;
        self.angular_velocities[index] = angular_velocity;
        self.colors[index] = color;
        self.particle_indexes.push_back(index);
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum EmitShape {
    Point, // The position of the particle system
    //Line(Vector2<f32>),
    //Rect(RectData),
    //Cone(ConeData),
    Circle(CircleData),
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct RectData {
    size: Vector2<f32>,
    spawn_type: SpawnType,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct ConeData {
    radius: f32,
    angle: f32,
    spawn_type: SpawnType,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct CircleData {
    radius: f32,
    spawn_type: SpawnType,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum SpawnType {
    Volume,
    Edge,
}

// decides how velocity should be calculated
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum VelocityType {
    //AlignToDirection(AlignToDirectionData),
    Angle(AngleData),
    Random,
}

#[derive(Clone, Copy)]
pub struct AlignToDirectionData {
    pub max_delta: Option<f32>,
}

#[derive(Clone, Copy)]
pub struct AngleData {
    pub angle: f32,
    pub max_delta: Option<f32>,
}

impl AngleData {
    pub fn new(angle: f32, max_delta: Option<f32>) -> Self {
        AngleData { angle, max_delta }
    }
}

impl EmitShape {
    // Todo: Implement other shapes other than point and cirle
    pub fn get_position(&self) -> Point2<f32> {
        match self {
            EmitShape::Point => Point2::new(0.0, 0.0),
            EmitShape::Circle(c) => {
                let mut dir = vec_from_angle(TAU);
                if let SpawnType::Volume = c.spawn_type {
                    dir *= rand::gen_range(0.0, 1.0);
                }
                dir.into()
            }
            //EmitShape::Line(v) => {Point2{x: 0.0, y: 0.0}},
            //EmitShape::Rect(r) => {Point2{x: 0.0, y: 0.0}},
            //EmitShape::Cone(c, a) => {Point2{x: 0.0, y: 0.0}},
        }
    }

    pub fn get_direction(
        &self,
        velocity_type: &VelocityType,
        _position: &Point2<f32>,
    ) -> Vector2<f32> {
        match velocity_type {
            VelocityType::Random => vec_from_angle(rand::gen_range(0.0, TAU)),
            VelocityType::Angle(a) => {
                let delta = match a.max_delta {
                    Some(d) => rand::gen_range(-d, d),
                    None => 0.0,
                };
                vec_from_angle(a.angle + delta)
            }
            // VelocityType::AlignToDirection(a) => {
            //     match self {
            //         EmitShape::Point => {
            //             vec_from_angle(TAU);
            //         },
            //         EmitShape::Circle(c) => {
            //             // From 0.0 out to where it will spawn

            //         },

            //     }
            // },
        }
    }
}

#[derive(Clone, Copy)]
pub enum ValueGetter<T> {
    Single(T),
    Range(T, T),
}

// Todo: Implement range, randomization
impl ValueGetter<ggez::graphics::Color> {
    pub fn get(&self) -> ggez::graphics::Color {
        match *self {
            ValueGetter::Single(v) => v,
            ValueGetter::Range(v1, v2) => {
                let (low_r, low_g, low_b) = v1.into();
                let (high_r, high_g, high_b) = v2.into();
                // gen_range doesn't support u8 in good-web-easy
                let r = rand::gen_range(low_r as i32, high_r as i32) as u8;
                let g = rand::gen_range(low_g as i32, high_g as i32) as u8;
                let b = rand::gen_range(low_b as i32, high_b as i32) as u8;
                (r, g, b).into()
            }
        }
    }
}

impl ValueGetter<f32> {
    pub fn get(&self) -> f32 {
        match *self {
            ValueGetter::Single(v) => v,
            ValueGetter::Range(v1, v2) => rand::gen_range(v1, v2),
        }
    }
    pub fn max(&self) -> f32 {
        match *self {
            ValueGetter::Single(v) => v,
            ValueGetter::Range(_v1, v2) => v2,
        }
    }
}

// Manage multiple systems
pub struct ParticleSystemCollection {
    particle_systems: HashMap<u32, ParticleSystem>,
    last_identifier: u32,
}

impl ParticleSystemCollection {
    pub fn new() -> Self {
        ParticleSystemCollection {
            particle_systems: HashMap::new(),
            last_identifier: 0,
        }
    }

    pub fn update(&mut self, delta: f32) {
        for (_identifier, system) in self.particle_systems.iter_mut() {
            system.update(delta);
        }
    }

    pub fn add_system(&mut self, system: ParticleSystem) -> u32 {
        self.last_identifier += 1;
        self.particle_systems.insert(self.last_identifier, system);
        self.last_identifier
    }

    pub fn get_mut(&mut self, identifier: u32) -> Option<&mut ParticleSystem> {
        if let Some(system) = self.particle_systems.get_mut(&identifier) {
            return Some(system);
        }
        None
    }

    // returns if system is still valid
    #[allow(dead_code)]
    pub fn emit(&mut self, system_identifier: u32, amount: i32) -> bool {
        if let Some(system) = self.particle_systems.get_mut(&system_identifier) {
            system.emit(amount);
            return true;
        }
        false
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for (_id, system) in self.particle_systems.iter_mut() {
            system.draw(ctx)?;
        }
        Ok(())
    }
}

pub fn emit_step_particle(
    particle_collection: &mut ParticleSystemCollection,
    step_id: &u32,
    amount: i32,
    is_right_dir: bool,
    position: &na::Point2<f32>,
    screen_size: &na::Point2<f32>,
) {
    let step_particle = particle_collection.get_mut(*step_id).unwrap();
    let mut pos_particle = na::Vector2::new(
        position.x as f32 / screen_size.x * 16.0,
        position.y as f32 / screen_size.x * 16.0,
    );

    if is_right_dir {
        step_particle.velocity_type = VelocityType::Angle(AngleData::new(-PI * 0.8, Some(0.2)));
    } else {
        step_particle.velocity_type = VelocityType::Angle(AngleData::new(PI * 0.8, Some(0.2)));
    }
    step_particle.scale = screen_size.x / 16.0;
    // offset to under player
    pos_particle.x += 16.0 * 0.5;
    pos_particle.y += 16.0;
    step_particle.position = pos_particle;

    step_particle.emit(amount);
}
