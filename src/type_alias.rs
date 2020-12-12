use dimensioned::si;
use dimensioned::typenum::Prod;

/// `f64` or `f32`
pub type Quantity = f64;

mod derived {
    use dimensioned::si::{self, SI};
    use dimensioned::{__derived_internal, derived};

    derived!(si, SI: GravityConstant = Meter3 / Kilogram / Second2);
}

pub type Unitless = si::Unitless<Quantity>;
pub type Kilogram = si::Kilogram<Quantity>;
pub type Meter = si::Meter<Quantity>;
pub type Second = si::Second<Quantity>;
pub type Meter2 = si::Meter2<Quantity>;
pub type Velocity = si::MeterPerSecond<Quantity>;
pub type Accel = si::MeterPerSecond2<Quantity>;
pub type KilogramMeter = Prod<Kilogram, Meter>;
pub type GravityConstant = derived::GravityConstant<Quantity>;
