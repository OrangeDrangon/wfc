use enum_map::Enum;
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
            .map(|(e1, e2)| e1 == e2)
            // accumulate a single boolean
            //
            // if this becomes a performance concern with large sockets one can return as soon as
            // they see false in the map step using more traditional for loops
            .fold(true, |acc, x| acc && x)
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
