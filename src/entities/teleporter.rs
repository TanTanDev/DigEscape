use crate::{sprite::SpriteComponent, transform_compontent::TransformComponent};
#[derive(Default)]
pub struct Teleporter {
    pub transform: TransformComponent,
    pub sprite: SpriteComponent,
}

#[derive(Default)]
pub struct Exit {
    pub transform: TransformComponent,
    pub sprite: SpriteComponent,
}
