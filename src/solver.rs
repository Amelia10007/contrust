use crate::state::State;
use crate::type_alias::Quantity;
use std::marker::PhantomData;
use std::ops::{AddAssign, Mul};

pub trait Solver {
    type State: State;

    fn progress(state: &mut Self::State, duration: <Self::State as State>::Axis);
}

pub struct ForwardEuler<S>(PhantomData<S>);

impl<S> Solver for ForwardEuler<S>
where
    S: State,
{
    type State = S;

    fn progress(state: &mut Self::State, duration: S::Axis) {
        let diff = state.calculate_difference();
        state.progress(duration, &diff);
    }
}

pub struct RungeKutta4<S>(PhantomData<S>);

impl<S> Solver for RungeKutta4<S>
where
    S: Clone + State + AddAssign<S>,
    S::Difference: Mul<S::Axis, Output = S>,
    S::Axis: Copy + Mul<Quantity, Output = S::Axis>,
{
    type State = S;

    fn progress(state: &mut Self::State, duration: S::Axis) {
        let k1 = state.calculate_difference();
        let mut state2 = state.clone();
        state2.progress(duration * 0.5, &k1);
        let k2 = state2.calculate_difference();
        let mut state3 = state.clone();
        state3.progress(duration * 0.5, &k2);
        let k3 = state3.calculate_difference();
        let mut state4 = state.clone();
        state4.progress(duration, &k3);
        let k4 = state4.calculate_difference();

        *state += k1 * (duration * (1.0 / 6.0));
        *state += k2 * (duration * (2.0 / 6.0));
        *state += k3 * (duration * (2.0 / 6.0));
        *state += k4 * (duration * (1.0 / 6.0));
    }
}
