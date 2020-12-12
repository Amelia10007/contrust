use crate::type_alias::*;
use pair_macro::Pair;

/// 空間内を移動できる質点．
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassPoint {
    pub mass: Kilogram,
    pub position: Pair<Meter>,
    pub velocity: Pair<Velocity>,
}

impl MassPoint {
    pub const fn new(mass: Kilogram, position: Pair<Meter>, velocity: Pair<Velocity>) -> MassPoint {
        Self {
            mass,
            position,
            velocity,
        }
    }
}
