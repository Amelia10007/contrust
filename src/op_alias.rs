use std::ops::{Add, Div, Mul, Sub};

pub trait AddSelf: Sized + Add<Output = Self> {}

impl<T> AddSelf for T where T: Add<Output = Self> {}

pub trait SubSelf: Sized + Sub<Output = Self> {}

impl<T> SubSelf for T where T: Sub<Output = Self> {}

pub trait MulScalar<S>: Sized + Mul<S, Output = Self> {}

impl<T, S> MulScalar<S> for T where T: Mul<S, Output = Self> {}

pub trait DivScalar<S>: Sized + Div<S, Output = Self> {}

impl<T, S> DivScalar<S> for T where T: Div<S, Output = Self> {}
