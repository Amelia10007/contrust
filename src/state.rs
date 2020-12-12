pub trait State {
    type Axis;
    type Difference;

    fn calculate_difference(&self) -> Self::Difference;

    fn progress(&mut self, duration: Self::Axis, diff: &Self::Difference);
}
