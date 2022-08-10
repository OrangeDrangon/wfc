use integer_sqrt::IntegerSquareRoot;

use crate::slots::{Location, Slot};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Pattern<Data> {
    pub pixel_data: Vec<Data>,
    size: usize,
}

impl<Data> Pattern<Data> {
    pub fn new(pixel_data: Vec<Data>) -> Self {
        let size = pixel_data.len().integer_sqrt();
        assert_eq!(pixel_data.len(), size * size);

        Self { size, pixel_data }
    }

    fn slot(&self, location: Location) -> Slot<Data> {
        match location {
            Location::North => Slot::new(
                self.pixel_data.iter().take(self.size).collect(),
                Location::North,
            ),
            Location::East => Slot::new(
                self.pixel_data
                    .iter()
                    .step_by(self.size)
                    .collect::<Vec<_>>(),
                Location::East,
            ),
            Location::South => Slot::new(
                self.pixel_data
                    .iter()
                    .skip(self.pixel_data.len() - self.size)
                    .collect(),
                Location::South,
            ),
            Location::West => Slot::new(
                self.pixel_data
                    .iter()
                    .skip(self.size - 1)
                    .step_by(self.size)
                    .collect::<Vec<_>>(),
                Location::West,
            ),
        }
    }
}

impl<Data: PartialEq> Pattern<Data> {
    pub(crate) fn is_compatible(&self, b: &Pattern<Data>, b_location: Location) -> bool {
        self.slot(b_location)
            .can_be_adjacent(&b.slot(b_location.opposite()))
    }
}

impl<Data: Clone + Default> Pattern<Data> {
    pub fn all_permutations(self) -> Vec<Self> {
        let mut patterns = Vec::with_capacity(8);

        let pattern = self;
        let reflected = pattern.reflect();
        let rotated = pattern.rotate();
        let rotated_reflected = rotated.reflect();
        let rotated_rotated = rotated.rotate();
        let rotated_rotated_reflected = rotated_rotated.reflect();
        let rotated_rotated_rotated = rotated_rotated.rotate();
        let rotated_rotated_rotated_reflected = rotated_rotated_rotated.reflect();

        patterns.push(pattern);
        patterns.push(reflected);
        patterns.push(rotated);
        patterns.push(rotated_reflected);
        patterns.push(rotated_rotated);
        patterns.push(rotated_rotated_reflected);
        patterns.push(rotated_rotated_rotated);
        patterns.push(rotated_rotated_rotated_reflected);

        patterns
    }

    pub fn rotate(&self) -> Self {
        Self {
            pixel_data: self.apply(|row, col, rotated| {
                rotated[row * self.size + col] = self.pixel_data[col * self.size + row].clone()
            }),
            size: self.size,
        }
    }

    pub fn reflect(&self) -> Self {
        Self {
            pixel_data: self.apply(|row, col, reflected| {
                reflected[row * self.size + col] =
                    self.pixel_data[row * self.size + self.size - 1 - col].clone()
            }),
            size: self.size,
        }
    }

    fn apply<F>(&self, f: F) -> Vec<Data>
    where
        F: Fn(usize, usize, &mut [Data]),
    {
        let mut out_data = vec![Data::default(); self.pixel_data.len()];

        for row in 0..self.size {
            for col in 0..self.size {
                f(row, col, &mut out_data)
            }
        }
        out_data
    }
}
