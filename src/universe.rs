use crate::grid::Grid;
use crate::grid_diff::*;
use dimensioned::si;
use wasm_bindgen::prelude::*;

mod derived_si {
    use dimensioned::si::{self, Kelvin, KilogramPerMeter3, Second, SI};
    use dimensioned::{__derived_internal, derived};

    derived!(si, SI: KelvinPerSecond = Kelvin / Second);
    derived!(
        si,
        SI: KilogramPerMeter3PerSecond = KilogramPerMeter3 / Second
    );
}

pub type Meter = si::Meter<f64>;
pub type Kelvin = si::Kelvin<f64>;
pub type Density = si::KilogramPerMeter3<f64>;
pub type Area = si::Meter2<f64>;
pub type Second = si::Second<f64>;
pub type Velocity = si::MeterPerSecond<f64>;
pub type SpecificHeat = si::JoulePerKilogramKelvin<f64>;
pub type ThermalConductivity = si::WattPerMeterKelvin<f64>;

type KelvinPerSecond = derived_si::KelvinPerSecond<f64>;
type KilogramPerMeter3PerSecond = derived_si::KilogramPerMeter3PerSecond<f64>;

#[derive(Debug, Default)]
struct MaterialProperty {
    gravity_constant: i32,
    specific_heat: SpecificHeat,
    thermal_conductivity: ThermalConductivity,
    cell_length: Meter,
    gas_constant: i32,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Universe {
    density_grid: Grid<Density>,
    velocity_x_grid: Grid<Velocity>,
    velocity_y_grid: Grid<Velocity>,
    temperature_grid: Grid<Kelvin>,
    property: MaterialProperty,
}

impl Universe {
    fn new(rows: usize, cols: usize) -> Universe {
        Self {
            density_grid: Grid::fill_default(rows, cols),
            velocity_x_grid: Grid::fill_default(rows, cols),
            velocity_y_grid: Grid::fill_default(rows, cols),
            temperature_grid: Grid::fill_default(rows, cols),
            property: Default::default(),
        }
    }

    fn tick(&mut self, duration: Second) {
        let delta = self.property.cell_length;
        let density_nabla = calculate_nabla(&self.density_grid, delta);
        let velocity_x_nabla = calculate_nabla(&self.velocity_x_grid, delta);
        let velocity_y_nabla = calculate_nabla(&self.velocity_y_grid, delta);
        let temperature_nabla = calculate_nabla(&self.temperature_grid, delta);
        let temperature_laplacian = calculate_laplacian(&self.temperature_grid, delta);

        let temperature_time_diff = {
            let coef = self.property.thermal_conductivity / self.property.specific_heat;
            temperature_laplacian.merge_entrywise(&self.density_grid, |&temperature, &density| {
                coef * temperature / density
            });
        };

        unimplemented!()
    }
}
