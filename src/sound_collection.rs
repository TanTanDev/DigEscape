use gwg as ggez;

use ggez::{audio, GameResult};

pub struct SoundCollection {
    pub sounds: [audio::Source; 10],
    pub is_on: bool,
}

impl SoundCollection {
    pub fn play(&mut self, index: usize) -> GameResult<()> {
        if !self.is_on {
            return Ok(());
        }
        if let Some(source) = self.sounds.get_mut(index) {
            source.play()?;
        }
        Err(ggez::error::GameError::SoundError)
    }
}
