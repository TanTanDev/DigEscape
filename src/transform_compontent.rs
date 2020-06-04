use nalgebra as na;
pub struct TransformComponent {
    pub position: na::Point2<i32>,
}

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            position: na::Point2::new(0, 0),
        }
    }
}
