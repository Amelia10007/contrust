use crate::grid::Grid;
use crate::op_alias::{AddSelf, DivScalar, MulScalar, SubSelf};
use dimensioned::typenum::{Prod, Quot};
use std::iter::once;
use std::ops::{Div, Mul};

pub fn calculate_partial_difference_x<T, U>(grid: &Grid<T>, delta: U) -> Grid<Quot<T, U>>
where
    T: Copy + SubSelf + Div<U> + DivScalar<f64>,
    U: Copy,
{
    grid.map_rowwise(|row| {
        let cols = row.cols();
        // 端の微分は一次精度
        let first = row[1] - row[0];
        let last = row[cols - 1] - row[cols - 2];
        // 端以外の微分は二次精度
        let inner = (1..=cols - 2)
            .map(move |x| row[x + 1] - row[x - 1])
            .map(move |diff| diff / 2.0);

        once(first)
            .chain(inner)
            .chain(once(last))
            .map(|diff| diff / delta)
    })
}

pub fn calculate_partial_difference_y<T, U>(grid: &Grid<T>, delta: U) -> Grid<Quot<T, U>>
where
    T: Copy + SubSelf + Div<U> + DivScalar<f64>,
    U: Copy,
{
    grid.map_colwise(|col| {
        // 端の微分は一次精度
        let rows = col.rows();
        let first = col[1] - col[0];
        let last = col[rows - 1] - col[rows - 2];
        // 端以外の微分は二次精度
        let inner = (1..=rows - 2)
            .map(move |y| col[y + 1] - col[y - 1])
            .map(move |diff| diff / 2.0);

        once(first)
            .chain(inner)
            .chain(once(last))
            .map(|diff| diff / delta)
    })
}

pub fn calculate_partial_difference_xx<T, U>(grid: &Grid<T>, delta: U) -> Grid<Quot<T, Prod<U, U>>>
where
    T: Copy + AddSelf + SubSelf + Div<Prod<U, U>> + MulScalar<f64>,
    U: Copy + Mul<U>,
{
    grid.map_rowwise(|row| {
        let cols = row.cols();
        // 端の微分
        let first = row[1] - row[0];
        let last = row[cols - 1] - row[cols - 2];
        // 端以外の微分
        let inner = (1..=cols - 2).map(move |x| row[x + 1] - row[x] * 2.0 + row[x - 1]);

        once(first)
            .chain(inner)
            .chain(once(last))
            .map(|diff| diff / (delta * delta))
    })
}

pub fn calculate_partial_difference_yy<T, U>(grid: &Grid<T>, delta: U) -> Grid<Quot<T, Prod<U, U>>>
where
    T: Copy + AddSelf + SubSelf + Div<Prod<U, U>> + MulScalar<f64>,
    U: Copy + Mul<U>,
{
    let (rows, cols) = grid.size();
    // 端の微分
    let first_row = (0..cols).map(|x| grid[1][x] - grid[0][x]);
    let last_row = (0..cols).map(|x| grid[rows - 1][x] - grid[rows - 2][x]);
    // 端以外の微分
    let inner_rows = (1..=rows - 2)
        .flat_map(|y| (0..cols).map(move |x| grid[y + 1][x] - grid[y][x] * 2.0 + grid[y - 1][x]));
    let grid_vec = first_row
        .chain(inner_rows)
        .chain(last_row)
        .map(|diff| diff / (delta * delta))
        .collect();
    Grid::from_vec(grid_vec, cols)
}

pub fn calculate_nabla<T, U>(grid: &Grid<T>, delta: U) -> (Grid<Quot<T, U>>, Grid<Quot<T, U>>)
where
    T: Copy + SubSelf + Div<U> + DivScalar<f64>,
    U: Copy,
{
    let x = calculate_partial_difference_x(grid, delta);
    let y = calculate_partial_difference_y(grid, delta);
    (x, y)
}

pub fn calculate_laplacian<T, U>(grid: &Grid<T>, delta: U) -> Grid<Quot<T, Prod<U, U>>>
where
    T: Copy + AddSelf + SubSelf + Div<Prod<U, U>> + MulScalar<f64>,
    U: Copy + Mul<U>,
    Quot<T, Prod<U, U>>: Copy + AddSelf,
{
    let xx = calculate_partial_difference_xx(grid, delta);
    let yy = calculate_partial_difference_yy(grid, delta);

    xx.merge_entrywise(&yy, |&x, &y| x + y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dimensioned::si::{Meter, PerMeter, Unitless, M};

    /// ↓row →col
    ///
    /// 0, 1, 3
    ///
    /// 3, 4, 6
    ///
    /// 7, 8, 10
    fn sample_grid() -> Grid<Meter<f64>> {
        let mut g = Grid::fill_default(3, 3);
        g[0][0] = 0.0 * M;
        g[0][1] = 1.0 * M;
        g[0][2] = 3.0 * M;
        g[1][0] = 3.0 * M;
        g[1][1] = 4.0 * M;
        g[1][2] = 6.0 * M;
        g[2][0] = 7.0 * M;
        g[2][1] = 8.0 * M;
        g[2][2] = 10.0 * M;

        g
    }

    #[test]
    fn test_calculate_partial_difference_x() {
        let g = sample_grid();
        let delta = Meter::new(10.0);

        let d = calculate_partial_difference_x(&g, delta);
        assert_eq!(g.size(), d.size());

        assert_eq!(Unitless::new(0.1), d[0][0]);
        assert_eq!(Unitless::new(0.15), d[0][1]);
        assert_eq!(Unitless::new(0.2), d[0][2]);
        assert_eq!(Unitless::new(0.1), d[1][0]);
        assert_eq!(Unitless::new(0.15), d[1][1]);
        assert_eq!(Unitless::new(0.2), d[1][2]);
        assert_eq!(Unitless::new(0.1), d[2][0]);
        assert_eq!(Unitless::new(0.15), d[2][1]);
        assert_eq!(Unitless::new(0.2), d[2][2]);
    }

    #[test]
    fn test_calculate_partial_difference_y() {
        let g = sample_grid();
        let delta = Meter::new(10.0);

        let d = calculate_partial_difference_y(&g, delta);
        assert_eq!(g.size(), d.size());

        assert_eq!(Unitless::new(0.3), d[0][0]);
        assert_eq!(Unitless::new(0.3), d[0][1]);
        assert_eq!(Unitless::new(0.3), d[0][2]);
        assert_eq!(Unitless::new(0.35), d[1][0]);
        assert_eq!(Unitless::new(0.35), d[1][1]);
        assert_eq!(Unitless::new(0.35), d[1][2]);
        assert_eq!(Unitless::new(0.4), d[2][0]);
        assert_eq!(Unitless::new(0.4), d[2][1]);
        assert_eq!(Unitless::new(0.4), d[2][2]);
    }

    #[test]
    fn test_calculate_partial_difference_xx() {
        let g = sample_grid();
        let delta = Meter::new(10.0);

        let d = calculate_partial_difference_xx(&g, delta);
        assert_eq!(g.size(), d.size());

        assert_eq!(PerMeter::new(0.01), d[0][0]);
        assert_eq!(PerMeter::new(0.01), d[0][1]);
        assert_eq!(PerMeter::new(0.02), d[0][2]);
        assert_eq!(PerMeter::new(0.01), d[1][0]);
        assert_eq!(PerMeter::new(0.01), d[1][1]);
        assert_eq!(PerMeter::new(0.02), d[1][2]);
        assert_eq!(PerMeter::new(0.01), d[2][0]);
        assert_eq!(PerMeter::new(0.01), d[2][1]);
        assert_eq!(PerMeter::new(0.02), d[2][2]);
    }

    #[test]
    fn test_calculate_partial_difference_yy() {
        let g = sample_grid();
        let delta = Meter::new(10.0);

        let d = calculate_partial_difference_yy(&g, delta);
        assert_eq!(g.size(), d.size());

        assert_eq!(PerMeter::new(0.03), d[0][0]);
        assert_eq!(PerMeter::new(0.03), d[0][1]);
        assert_eq!(PerMeter::new(0.03), d[0][2]);
        assert_eq!(PerMeter::new(0.01), d[1][0]);
        assert_eq!(PerMeter::new(0.01), d[1][1]);
        assert_eq!(PerMeter::new(0.01), d[1][2]);
        assert_eq!(PerMeter::new(0.04), d[2][0]);
        assert_eq!(PerMeter::new(0.04), d[2][1]);
        assert_eq!(PerMeter::new(0.04), d[2][2]);
    }

    #[test]
    fn test_calculate_nabla() {
        let g = sample_grid();
        let delta = Meter::new(10.0);

        let (x, y) = calculate_nabla(&g, delta);

        assert_eq!(x, calculate_partial_difference_x(&g, delta));
        assert_eq!(y, calculate_partial_difference_y(&g, delta));
    }

    #[test]
    fn test_calculate_laplacian() {
        let g = sample_grid();
        let delta = Meter::new(10.0);

        let d = calculate_laplacian(&g, delta);
        assert_eq!(g.size(), d.size());

        assert_eq!(PerMeter::new(0.01 + 0.03), d[0][0]);
        assert_eq!(PerMeter::new(0.01 + 0.03), d[0][1]);
        assert_eq!(PerMeter::new(0.02 + 0.03), d[0][2]);
        assert_eq!(PerMeter::new(0.01 + 0.01), d[1][0]);
        assert_eq!(PerMeter::new(0.01 + 0.01), d[1][1]);
        assert_eq!(PerMeter::new(0.02 + 0.01), d[1][2]);
        assert_eq!(PerMeter::new(0.01 + 0.04), d[2][0]);
        assert_eq!(PerMeter::new(0.01 + 0.04), d[2][1]);
        assert_eq!(PerMeter::new(0.02 + 0.04), d[2][2]);
    }
}
