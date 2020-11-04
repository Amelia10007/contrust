use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Grid<T> {
    v: Vec<T>,
    cols: usize,
}

impl<T> Grid<T> {
    pub fn from_vec(v: Vec<T>, cols: usize) -> Grid<T> {
        assert!(v.len() % cols == 0);

        Self { v, cols }
    }

    pub fn rows(&self) -> usize {
        self.v.len() / self.cols()
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Returns `(rows, cols)`
    pub fn size(&self) -> (usize, usize) {
        (self.rows(), self.cols())
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = Row<'_, T>> {
        (0..self.rows()).map(move |row| Row { grid: self, row })
    }

    pub fn iter_cols(&self) -> impl Iterator<Item = Col<'_, T>> {
        (0..self.cols()).map(move |col| Col { grid: self, col })
    }

    pub fn map_rowwise<'g, U, I, F>(&'g self, f: F) -> Grid<U>
    where
        F: FnMut(Row<'g, T>) -> I,
        I: IntoIterator<Item = U> + 'g,
    {
        let v = self.iter_rows().flat_map(f).collect::<Vec<_>>();
        debug_assert_eq!(self.v.len(), v.len());
        Grid { v, cols: self.cols }
    }

    pub fn map_colwise<'g, U, I, F>(&'g self, f: F) -> Grid<U>
    where
        F: FnMut(Col<'g, T>) -> I,
        I: IntoIterator<Item = U> + 'g,
    {
        let mut g = {
            use std::mem::MaybeUninit;

            let (rows, cols) = self.size();
            let v = (0..rows * cols)
                .map(|_| MaybeUninit::uninit())
                .map(|x| unsafe { x.assume_init() })
                .collect();

            Grid::from_vec(v, cols)
        };

        for (x, col) in self.iter_cols().map(f).enumerate() {
            for (y, item) in col.into_iter().enumerate() {
                g[y][x] = item;
            }
        }

        g
    }

    pub fn merge_entrywise<U, V, F>(&self, right: &Grid<U>, mut f: F) -> Grid<V>
    where
        F: FnMut(&T, &U) -> V,
    {
        assert_eq!(self.size(), right.size());

        let v = self
            .v
            .iter()
            .zip(right.v.iter())
            .map(|(left, right)| f(left, right))
            .collect();
        Grid { v, cols: self.cols }
    }
}

impl<T: Clone + Default> Grid<T> {
    pub fn fill_default(rows: usize, cols: usize) -> Grid<T> {
        let v = vec![T::default(); rows * cols];
        Self { v, cols }
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        let start = index * self.cols;
        let end = start + self.cols;
        &self.v[start..end]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let start = index * self.cols;
        let end = start + self.cols;
        &mut self.v[start..end]
    }
}

pub struct Row<'g, T> {
    grid: &'g Grid<T>,
    row: usize,
}

impl<'g, T> Row<'g, T> {
    pub fn row(&self) -> usize {
        self.row
    }

    pub fn cols(&self) -> usize {
        self.grid.cols()
    }

    pub fn into_iter(self) -> impl Iterator<Item = &'g T> + 'g {
        (0..self.grid.cols()).map(move |x| &self.grid[self.row][x])
    }
}

impl<'g, T> Index<usize> for Row<'g, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[self.row][index]
    }
}
pub struct Col<'g, T> {
    grid: &'g Grid<T>,
    col: usize,
}

impl<'g, T> Col<'g, T> {
    pub fn rows(&self) -> usize {
        self.grid.rows()
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn into_iter(self) -> impl Iterator<Item = &'g T> + 'g {
        (0..self.grid.rows()).map(move |y| &self.grid[y][self.col])
    }
}

impl<'g, T> Index<usize> for Col<'g, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[index][self.col]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ↓row →col
    ///
    /// 0, 1, 2
    ///
    /// 3, 4, 5
    fn sample_grid() -> Grid<i32> {
        let mut g = Grid::fill_default(2, 3);
        g[0][0] = 0;
        g[0][1] = 1;
        g[0][2] = 2;
        g[1][0] = 3;
        g[1][1] = 4;
        g[1][2] = 5;
        g
    }

    // #[test]
    // fn test_from_rows() {
    //     let rows = vec![vec![0, 1, 2], vec![3, 4, 5]];
    //     let g = Grid::from_rows(rows.into_iter().map(|r| r.into_iter()));
    //     assert_eq!(sample_grid(), g);
    // }

    // #[test]
    // fn test_from_cols() {
    //     let cols = vec![vec![0, 3], vec![1, 4], vec![2, 5]];
    //     let g = Grid::from_cols(cols.into_iter().map(|r| r.into_iter()));
    //     assert_eq!(sample_grid(), g);
    // }

    #[test]
    fn test_rows() {
        assert_eq!(2, sample_grid().rows());
    }

    #[test]
    fn test_cols() {
        assert_eq!(3, sample_grid().cols());
    }

    #[test]
    fn test_size() {
        assert_eq!((2, 3), sample_grid().size());
    }

    #[test]
    fn test_iter_rows() {
        let g = sample_grid();
        let mut r = g.iter_rows();

        assert_eq!(
            vec![0, 1, 2],
            r.next().unwrap().into_iter().copied().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![3, 4, 5],
            r.next().unwrap().into_iter().copied().collect::<Vec<_>>()
        );
        assert!(r.next().is_none())
    }

    #[test]
    fn test_iter_cols() {
        let g = sample_grid();
        let mut c = g.iter_cols();

        assert_eq!(
            vec![0, 3],
            c.next().unwrap().into_iter().copied().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![1, 4],
            c.next().unwrap().into_iter().copied().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![2, 5],
            c.next().unwrap().into_iter().copied().collect::<Vec<_>>()
        );
        assert!(c.next().is_none())
    }

    #[test]
    fn test_map_rowwise() {
        let g = sample_grid();
        let gg = g.map_rowwise(move |row| {
            let r = row.row() as i32;
            row.into_iter().map(move |&i| i + r)
        });

        assert_eq!(0 + 0, gg[0][0]);
        assert_eq!(1 + 0, gg[0][1]);
        assert_eq!(2 + 0, gg[0][2]);
        assert_eq!(3 + 1, gg[1][0]);
        assert_eq!(4 + 1, gg[1][1]);
        assert_eq!(5 + 1, gg[1][2]);
    }

    #[test]
    fn test_map_colwise() {
        let g = sample_grid();
        let gg = g.map_colwise(move |col| {
            let c = col.col() as i32;
            col.into_iter().map(move |&i| i + c)
        });

        assert_eq!(0 + 0, gg[0][0]);
        assert_eq!(3 + 0, gg[1][0]);
        assert_eq!(1 + 1, gg[0][1]);
        assert_eq!(4 + 1, gg[1][1]);
        assert_eq!(2 + 2, gg[0][2]);
        assert_eq!(5 + 2, gg[1][2]);
    }
}
