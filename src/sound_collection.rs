use gwg as ggez;

use ggez::audio;

pub struct SoundCollection {
    pub sounds: [audio::Source; 10],
    pub is_on: bool,
}

impl SoundCollection {
    pub fn play(&mut self, index: usize) {
        if !self.is_on {
            return;
        }
        if let Some(source) = self.sounds.get_mut(index) {
            let _ = source.play();
        }
    }
}
