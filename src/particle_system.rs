use std::collections::VecDeque;
use mint;
use mint::{Point2, Vector2};

static DEFAULT_CAPACITY: usize = 8;

struct ParticleSystem {
    // Particle data
    positions: Vec<Point2::<f32>>,
    velocities: Vec<Vector2::<f32>>,
    scales: Vec<Vector2::<f32>>,
    ages: Vec<f32>,
    particle_indexes: Vec<i32>,
    available_indexes: VecDeque<usize>,

    // System data
    emit_shape: EmitShape,
    start_speed: ValueGetter,
}

impl ParticleSystem {
    pub fn new() -> Self {
        let mut available_indexes = VecDeque::with_capacity(DEFAULT_CAPACITY); 
        for i in 0 .. available_indexes.len() {
            available_indexes.push_back(i);
        }

        ParticleSystem {
            positions: Vec::with_capacity(DEFAULT_CAPACITY),
            velocities: Vec::with_capacity(DEFAULT_CAPACITY),
            scales: Vec::with_capacity(DEFAULT_CAPACITY),
            ages: Vec::with_capacity(DEFAULT_CAPACITY),
            particle_indexes: Vec::with_capacity(DEFAULT_CAPACITY),
            available_indexes,
            emit_shape: EmitShape::Point(Point2{x:0.0, y:0.0}),
            start_speed: ValueGetter::Single(1.0),
        }
    }

    pub fn update(&mut self, dt: f32) {

    }

    pub fn emit(&mut self, amount: i32) {
        // Get indeces
        for i in 0..amount {
            let index_option = self.available_indexes.pop_front(); 
            match index_option {
                Some(index) => {
                    // make unused particle come alive
                    self.particle_setup(index);
                },
                None => {
                    // Resize vectors and spawn a new particle
                }
            }
        }
    }

    // Setup the data for a newly created particle
    fn particle_setup(&mut self, index: usize) {
        let age_option = self.ages.get_mut(index);
        if let Some(age) = age_option {
            *age = 0.0;
        }
        let speed = self.start_speed.get();
    }
}

enum EmitShape {
    Point(Point2<f32>),
    Line(Vector2<f32>),
    Rect(Vector2<f32>),
    Cone(Vector2<f32>, f32), // Dir, angle delta
}

impl EmitShape {
    // Todo: Implement other shapes other than point
    pub fn get_position(&self) -> Point2::<f32>{
        match self {
            EmitShape::Point(p) => {*p},
            EmitShape::Line(v) => {Point2{x: 0.0, y: 0.0}},
            EmitShape::Rect(r) => {Point2{x: 0.0, y: 0.0}},
            EmitShape::Cone(c, a) => {Point2{x: 0.0, y: 0.0}},
        }
    }
}

enum ValueGetter {
    Single(f32),
    Range(f32, f32),
}

// Todo: Implement range, randomization
impl ValueGetter {
    pub fn get(&self) -> f32 {
        match self {
            ValueGetter::Single(v) => *v,
            ValueGetter::Range(v1, v2) => *v1,
        }
    }
}

// Manage multiple systems
struct ParticleSystemCollection {
    particle_systems: Vec<ParticleSystem>,
}
