use crate::mass::MassPoint;
use crate::solver::{RungeKutta4, Solver};
use crate::state::State;
use crate::type_alias::*;
use dimensioned::typenum::P3;
use dimensioned::{Root, Sqrt};
use itertools::{Either, Itertools};
use pair_macro::Pair;
use std::ops::{AddAssign, Mul};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Universe {
    gravity_constant: GravityConstant,

    ms: Vec<Kilogram>,
    xs: Vec<Meter>,
    ys: Vec<Meter>,
    us: Vec<Velocity>,
    vs: Vec<Velocity>,

    minimum_ratio_for_integration: Unitless,
}

impl Universe {
    fn masses(&self) -> impl Iterator<Item = MassPoint> + '_ {
        self.ms
            .iter()
            .zip(self.xs.iter())
            .zip(self.ys.iter())
            .zip(self.us.iter())
            .zip(self.vs.iter())
            .map(|((((&m, &x), &y), &u), &v)| MassPoint::new(m, Pair::new(x, y), Pair::new(u, v)))
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        crate::utils::set_panic_hook();

        Self {
            gravity_constant: GravityConstant::new(1.0),
            ms: vec![],
            xs: vec![],
            ys: vec![],
            us: vec![],
            vs: vec![],
            minimum_ratio_for_integration: Unitless::new(1.0),
        }
    }

    pub fn mass_count(&self) -> usize {
        self.ms.len()
    }

    pub fn mass_ptr(&self) -> *const Quantity {
        self.ms.as_ptr().cast()
    }

    pub fn position_x_ptr(&self) -> *const Quantity {
        self.xs.as_ptr().cast()
    }

    pub fn position_y_ptr(&self) -> *const Quantity {
        self.ys.as_ptr().cast()
    }

    pub fn velocity_x_ptr(&self) -> *const Quantity {
        self.us.as_ptr().cast()
    }

    pub fn velocity_y_ptr(&self) -> *const Quantity {
        self.vs.as_ptr().cast()
    }

    pub fn tick(&mut self, duration_second: Quantity) {
        RungeKutta4::progress(self, Second::new(duration_second));
        //ForwardEuler::progress(self, Second::new(duration_second));

        let density = Density::new(5.0);
        let masses = merge_masspoints(self.masses().collect(), density);

        self.ms = masses.iter().map(|mp| mp.mass).collect();
        self.xs = masses.iter().map(|mp| mp.position.x).collect();
        self.ys = masses.iter().map(|mp| mp.position.y).collect();
        self.us = masses.iter().map(|mp| mp.velocity.x).collect();
        self.vs = masses.iter().map(|mp| mp.velocity.y).collect();
    }

    pub fn add_mass(&mut self, m: Quantity, x: Quantity, y: Quantity, u: Quantity, v: Quantity) {
        self.ms.push(Kilogram::new(m));
        self.xs.push(Meter::new(x));
        self.ys.push(Meter::new(y));
        self.us.push(Velocity::new(u));
        self.vs.push(Velocity::new(v));
    }

    pub fn set_minimum_ratio_for_integration(&mut self, ratio: Quantity) {
        self.minimum_ratio_for_integration = Unitless::new(ratio);
    }
}

impl State for Universe {
    type Axis = Second;
    type Difference = UniverseDiff;

    fn calculate_difference(&self) -> Self::Difference {
        let velocities = self
            .us
            .iter()
            .zip(self.vs.iter())
            .map(|(&u, &v)| Pair::new(u, v))
            .collect();

        let gravity_cutoff = Meter::new(2.5);

        let mut accels = vec![Default::default(); self.mass_count()];
        crate::gravity_calc::calculate_accels(
            &self.masses().collect::<Vec<_>>(),
            &mut accels,
            self.gravity_constant,
            self.minimum_ratio_for_integration,
            gravity_cutoff,
        );

        UniverseDiff { velocities, accels }
    }

    fn progress(&mut self, duration: Second, diff: &Self::Difference) {
        // update position
        self.xs
            .iter_mut()
            .zip(diff.velocities.iter().map(|&v| v.x))
            .for_each(|(x, u)| *x += u * duration);
        self.ys
            .iter_mut()
            .zip(diff.velocities.iter().map(|&v| v.y))
            .for_each(|(y, v)| *y += v * duration);
        // update velocity
        self.us
            .iter_mut()
            .zip(diff.accels.iter().map(|&a| a.x))
            .for_each(|(u, a)| *u += a * duration);
        self.vs
            .iter_mut()
            .zip(diff.accels.iter().map(|&a| a.y))
            .for_each(|(v, a)| *v += a * duration);
    }
}

impl AddAssign for Universe {
    fn add_assign(&mut self, rhs: Self) {
        self.xs
            .iter_mut()
            .zip(rhs.xs.iter())
            .for_each(|(l, &r)| *l += r);
        self.ys
            .iter_mut()
            .zip(rhs.ys.iter())
            .for_each(|(l, &r)| *l += r);
        self.us
            .iter_mut()
            .zip(rhs.us.iter())
            .for_each(|(l, &r)| *l += r);
        self.vs
            .iter_mut()
            .zip(rhs.vs.iter())
            .for_each(|(l, &r)| *l += r);
    }
}

pub struct UniverseDiff {
    velocities: Vec<Pair<Velocity>>,
    accels: Vec<Pair<Accel>>,
}

impl Mul<Second> for UniverseDiff {
    type Output = Universe;

    fn mul(self, rhs: Second) -> Self::Output {
        let xs = self.velocities.iter().map(|&v| v.x * rhs).collect();
        let ys = self.velocities.iter().map(|&v| v.y * rhs).collect();
        let us = self.accels.iter().map(|&a| a.x * rhs).collect();
        let vs = self.accels.iter().map(|&a| a.y * rhs).collect();
        Universe {
            gravity_constant: GravityConstant::default(),
            ms: vec![],
            xs,
            ys,
            us,
            vs,
            minimum_ratio_for_integration: Unitless::default(),
        }
    }
}

fn merge_masspoints(mut mass_points: Vec<MassPoint>, density: Density) -> Vec<MassPoint> {
    let mut result = vec![];

    while let Some(temp) = mass_points.pop() {
        let (conflicts, unconflicts): (Vec<_>, Vec<_>) =
            mass_points.into_iter().partition_map(|mp| {
                if conflicts(mp, temp, density) {
                    Either::Left(mp)
                } else {
                    Either::Right(mp)
                }
            });

        let merged = conflicts
            .into_iter()
            .fold(temp, |merged, m| merge(merged, m));

        result.push(merged);

        mass_points = unconflicts;
    }

    result
}

fn conflicts(mp1: MassPoint, mp2: MassPoint, density: Density) -> bool {
    let r1 = (mp1.mass / density).root(P3::new());
    let r2 = (mp2.mass / density).root(P3::new());

    let distance = (mp1.position - mp2.position)
        .into_iter()
        .fold(Meter2::new(0.0), |acc, cur| acc + cur * cur)
        .sqrt();

    distance < r1 + r2
}

fn merge(a: MassPoint, b: MassPoint) -> MassPoint {
    let mass = a.mass + b.mass;
    let position = (a.position * a.mass + b.position * b.mass) / mass;
    let velocity = (a.velocity * a.mass + b.velocity * b.mass) / mass;
    MassPoint::new(mass, position, velocity)
}
