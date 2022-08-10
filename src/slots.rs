use std::ops::{Deref, DerefMut};

use enum_map::{Enum, EnumMap};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Enum)]
pub enum Location {
    North,
    East,
    South,
    West,
}

impl Location {
    pub fn opposite(&self) -> Self {
        match self {
            Location::North => Location::South,
            Location::East => Location::West,
            Location::South => Location::North,
            Location::West => Location::East,
        }
    }

    pub fn rotate(&self) -> Self {
        match self {
            Location::North => Location::East,
            Location::East => Location::South,
            Location::South => Location::West,
            Location::West => Location::North,
        }
    }

    pub fn reflect(&self) -> Self {
        match self {
            Location::North => Location::North,
            Location::East => Location::West,
            Location::South => Location::South,
            Location::West => Location::East,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct LocationTable<T>(EnumMap<Location, T>);

impl<T> Deref for LocationTable<T> {
    type Target = EnumMap<Location, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for LocationTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot<'a, Data> {
    data: Vec<&'a Data>,
    location: Location,
}

impl<'a, Data> Slot<'a, Data> {
    pub(crate) fn new(data: Vec<&'a Data>, location: Location) -> Self {
        Self { data, location }
    }
}

impl<'a, Data> Slot<'a, Data>
where
    Data: PartialEq,
{
    pub fn can_be_adjacent(&self, slot: &Slot<Data>) -> bool {
        if self.location != slot.location.opposite() {
            return false;
        }

        if self.data.len() != slot.data.len() {
            return false;
        }

        self.data
            // combine the two iterators element by element into tuple
            .iter()
            .zip(slot.data.iter().rev())
            // compare each element to its partner in the same location
            .all(|(e1, e2)| e1 == e2)
    }

    #[cfg(test)]
    pub(crate) fn data_eq(&self, b: &Vec<&'a Data>) -> bool {
        &self.data == b
    }
}

#[cfg(test)]
mod test {
    use super::*;
    mod slot {

        use super::*;

        #[test]
        fn can_be_adjacent() {
            assert_eq!(
                true,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::South))
            );
            assert_eq!(
                true,
                Slot::new(vec![2, 2, 3, 1].iter().collect(), Location::East).can_be_adjacent(
                    &Slot::new(vec![1, 3, 2, 2].iter().collect(), Location::West)
                )
            );
            assert_eq!(
                true,
                Slot::new(vec![2, 2, 3, 1].iter().collect(), Location::South).can_be_adjacent(
                    &Slot::new(vec![1, 3, 2, 2].iter().collect(), Location::North)
                )
            );
            assert_eq!(
                true,
                Slot::new(vec![2, 2, 3, 1].iter().collect(), Location::East).can_be_adjacent(
                    &Slot::new(vec![1, 3, 2, 2].iter().collect(), Location::West)
                )
            );
            assert_eq!(
                true,
                Slot::new(vec![2, 2, 3, 1].iter().collect(), Location::West).can_be_adjacent(
                    &Slot::new(vec![1, 3, 2, 2].iter().collect(), Location::East)
                )
            );

            assert_eq!(
                false,
                Slot::new(vec![1, 2, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::South))
            );
            assert_eq!(
                false,
                Slot::new(vec![2, 2, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![2, 2, 1].iter().collect(), Location::South))
            );

            assert_eq!(
                false,
                Slot::new(vec![1, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::South))
            );

            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::North))
            );
            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::East)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::East))
            );
            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::South)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::South))
            );
            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::West)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::West))
            );

            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::East))
            );
            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::North)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::West))
            );
            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::South)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::East))
            );
            assert_eq!(
                false,
                Slot::new(vec![1, 1, 1].iter().collect(), Location::South)
                    .can_be_adjacent(&Slot::new(vec![1, 1, 1].iter().collect(), Location::West))
            );
        }
    }
}
