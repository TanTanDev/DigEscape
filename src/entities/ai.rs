#[derive(PartialEq)]
pub enum AiState {
    Walk,
    Attack,
}

pub struct AiComponent {
    pub state: AiState,
    pub turn_taken: bool,
}

impl Default for AiComponent {
    fn default() -> Self {
        AiComponent {
            state: AiState::Attack,
            turn_taken: false,
        }
    }
}
