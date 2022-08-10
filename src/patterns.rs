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

#[cfg(test)]
mod test {
    use super::*;
    use enum_map::EnumMap;

    #[test]
    fn rotate() {
        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9
        ];

        #[rustfmt::skip]
        let expected: Vec<usize> = vec![
            7, 4, 1,
            8, 5, 2,
            9, 6, 3
        ];

        let pattern = Pattern::new(data);

        assert_eq!(expected, pattern.rotate().pixel_data);

        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3, 4, 
            5, 6, 7, 8, 
            9, 10, 11, 12,
            13, 14, 15, 16
        ];

        #[rustfmt::skip]
        let expected: Vec<usize> = vec![
            13, 9, 5, 1,
            14, 10, 6, 2,
            15, 11, 7, 3,
            16, 12, 8, 4
        ];

        let pattern = Pattern::new(data);

        assert_eq!(expected, pattern.rotate().pixel_data);
    }

    #[test]
    fn reflect() {
        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9
        ];

        #[rustfmt::skip]
        let expected: Vec<usize> = vec![
            3, 2, 1,
            6, 5, 4,
            9, 8, 7
        ];

        let pattern = Pattern::new(data);

        assert_eq!(expected, pattern.reflect().pixel_data);

        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3, 4, 
            5, 6, 7, 8, 
            9, 10, 11, 12,
            13, 14, 15, 16
        ];

        #[rustfmt::skip]
        let expected: Vec<usize> = vec![
            4, 3, 2, 1,
            8, 7, 6, 5,
            12, 11, 10, 9,
            16, 15, 14, 13
        ];

        let pattern = Pattern::new(data);

        assert_eq!(expected, pattern.reflect().pixel_data);
    }

    #[test]
    fn north_slot() {
        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9
        ];

        #[rustfmt::skip]
        let slots: EnumMap<Location, Vec<usize>> = EnumMap::from_array([
            vec![1, 2, 3],
            vec![3, 6, 9],
            vec![7, 8, 9],
            vec![1, 4, 7],
        ]);

        let pattern = Pattern::new(data);

        let slot = pattern.slot(Location::North);

        assert!(slot.data_eq(&slots[Location::North].iter().collect()))
    }

    #[test]
    fn east_slot() {
        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9
        ];

        #[rustfmt::skip]
        let slots: EnumMap<Location, Vec<usize>> = EnumMap::from_array([
            vec![1, 2, 3],
            vec![3, 6, 9],
            vec![7, 8, 9],
            vec![1, 4, 7],
        ]);

        let pattern = Pattern::new(data);

        let slot = pattern.slot(Location::East);

        assert!(slot.data_eq(&slots[Location::East].iter().collect()))
    }

    #[test]
    fn south_slot() {
        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9
        ];

        #[rustfmt::skip]
        let slots: EnumMap<Location, Vec<usize>> = EnumMap::from_array([
            vec![1, 2, 3],
            vec![3, 6, 9],
            vec![7, 8, 9],
            vec![1, 4, 7],
        ]);

        let pattern = Pattern::new(data);

        let slot = pattern.slot(Location::South);

        assert!(slot.data_eq(&slots[Location::South].iter().collect()))
    }

    #[test]
    fn west_slot() {
        #[rustfmt::skip]
        let data: Vec<usize> = vec![
            1, 2, 3,
            4, 5, 6,
            7, 8, 9
        ];

        #[rustfmt::skip]
        let slots: EnumMap<Location, Vec<usize>> = EnumMap::from_array([
            vec![1, 2, 3],
            vec![3, 6, 9],
            vec![7, 8, 9],
            vec![1, 4, 7],
        ]);

        let pattern = Pattern::new(data);

        let slot = pattern.slot(Location::West);

        assert!(slot.data_eq(&slots[Location::West].iter().collect()))
    }
}
