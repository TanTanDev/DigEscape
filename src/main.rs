mod constantes;
mod sprite;
mod transform_compontent;

mod entities;
mod map;
mod particle_system;
mod sound_collection;
mod util;

mod states;
use states::main_state::MainState;

// Magic!
use gwg as ggez;

use ggez::GameResult;

fn main() -> GameResult {
    ggez::start(
        ggez::conf::Conf {
            cache: ggez::conf::Cache::Tar(include_bytes!("resources.tar").to_vec()),
            loading: ggez::conf::Loading::Embedded,
            ..Default::default()
        }, // conf
        |mut context| Box::new(MainState::new(&mut context).unwrap()),
    ) // ggez::start
}
