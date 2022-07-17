mod slots;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

use enum_map::EnumMap;
use hashbag::HashBag;
use integer_sqrt::IntegerSquareRoot;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use slots::{Location, Slot};
use strum::IntoEnumIterator;

type LocationTable<T> = EnumMap<Location, T>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Pattern<Data> {
    pixel_data: Vec<Data>,
    size: usize,
}

impl<Data> Pattern<Data> {
    pub fn new(pixel_data: Vec<Data>) -> Self {
        Self {
            size: pixel_data.len().integer_sqrt(),
            pixel_data,
        }
    }

    pub fn slot(&self, location: Location) -> Slot<Data> {
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

impl<Data> Pattern<Data>
where
    Data: Clone + Default,
{
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

#[derive(Debug, Clone)]
pub struct PatternAdjacencyMap<'a, Data> {
    adjacency: HashMap<&'a Pattern<Data>, LocationTable<HashSet<&'a Pattern<Data>>>>,
}

impl<'a, Data> PatternAdjacencyMap<'a, Data>
where
    Data: Eq + Hash,
{
    pub fn new<T: Iterator<Item = &'a Pattern<Data>> + Clone>(patterns: T) -> Self {
        let mut adjacency: HashMap<&'a Pattern<Data>, LocationTable<HashSet<&'a Pattern<Data>>>> =
            HashMap::default();

        for pattern in patterns.clone() {
            for neighbor in patterns.clone() {
                for location in Location::iter() {
                    let pattern_slot = pattern.slot(location);
                    let neighbor_slot = neighbor.slot(location.opposite());

                    if pattern_slot.can_be_adjacent(&neighbor_slot) {
                        let enum_map = adjacency.entry(pattern).or_default();

                        let bag = &mut enum_map[location];
                        bag.insert(neighbor);
                    }
                }
            }
        }

        Self { adjacency }
    }
}

impl<'a, Data> Index<&'a Pattern<Data>> for PatternAdjacencyMap<'a, Data>
where
    Data: Eq + Hash,
{
    type Output = LocationTable<HashSet<&'a Pattern<Data>>>;

    fn index(&self, index: &'a Pattern<Data>) -> &Self::Output {
        &self.adjacency[index]
    }
}

impl<'a, Data> IndexMut<&'a Pattern<Data>> for PatternAdjacencyMap<'a, Data>
where
    Data: Eq + Hash,
{
    fn index_mut(&mut self, index: &'a Pattern<Data>) -> &mut Self::Output {
        self.adjacency.get_mut(index).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Tile<'a, Data>
where
    Data: Hash,
{
    pattern: &'a Pattern<Data>,
    adjacency: &'a LocationTable<HashSet<&'a Pattern<Data>>>,
    probability: f64,
}

impl<'a, Data> Tile<'a, Data>
where
    Data: Hash,
{
    pub fn new(
        pattern: &'a Pattern<Data>,
        adjacency: &'a LocationTable<HashSet<&'a Pattern<Data>>>,
        probability: f64,
    ) -> Self {
        Self {
            pattern,
            adjacency,
            probability,
        }
    }
}

impl<'a, Data> Tile<'a, Data>
where
    Data: Hash + Eq,
{
    pub fn agree(&self, neighbor_location: Location, neighbor: &Tile<'a, Data>) -> bool {
        let location_data = &self.adjacency[neighbor_location];

        location_data.contains(neighbor.pattern)
    }
}

#[derive(Debug, Clone)]
pub struct Cell<'a, Data>
where
    Data: Hash,
{
    tiles: Vec<&'a Tile<'a, Data>>,
    ways_to_become_pattern: PatternAdjacencyMap<'a, Data>,
}

impl<'a, Data> Cell<'a, Data>
where
    Data: Hash + Eq,
{
    pub fn new(
        tiles: Vec<&'a Tile<'a, Data>>,
        ways_to_become_pattern: PatternAdjacencyMap<'a, Data>,
    ) -> Self {
        Self {
            tiles,
            ways_to_become_pattern,
        }
    }

    pub fn collapsed(&self) -> bool {
        self.tiles.len() == 1
    }

    pub fn entropy(&self) -> f64 {
        -self
            .tiles
            .iter()
            .map(|tile| tile.probability * tile.probability.log(2.0))
            .fold(0.0, |shannons, shannon| shannons + shannon)
    }

    fn collapse(&mut self) -> Vec<&'a Tile<'a, Data>> {
        if !self.collapsed() {
            let dist = WeightedIndex::new(self.tiles.iter().map(|tile| tile.probability)).unwrap();
            let index = dist.sample(&mut rand::thread_rng());

            let collapsed_to = self.tiles.remove(index);

            let out = std::mem::take(&mut self.tiles);

            self.tiles = vec![collapsed_to];

            out
        } else {
            vec![]
        }
    }

    fn removed_neighbor_tile(&mut self, neighbor_location: Location, removed: &Tile<'a, Data>) {
        let mut patterns_to_remove = vec![];
        for (pattern, location_map) in &mut self.ways_to_become_pattern.adjacency {
            {
                let possible = &mut location_map[neighbor_location];
                possible.remove(removed.pattern);
            }

            let count = (&location_map[Location::North])
                .intersection(&location_map[Location::East])
                .cloned()
                .collect::<HashSet<_>>()
                .intersection(&location_map[Location::South])
                .cloned()
                .collect::<HashSet<_>>()
                .intersection(&location_map[Location::West])
                .count();

            if count == 0 {
                patterns_to_remove.push(pattern.clone());
            }
        }

        for pattern in patterns_to_remove.into_iter() {
            self.ways_to_become_pattern.adjacency.remove(pattern);
            self.tiles.retain(|e| e.pattern != pattern);
        }

        dbg!(self.tiles.len());
    }
}

struct RemovedTile<'a, Data>
where
    Data: Hash,
{
    cell_index: usize,
    tile: &'a Tile<'a, Data>,
}

pub struct Wave<'a, Data>
where
    Data: Hash,
{
    pub cells: Vec<Cell<'a, Data>>,
    x_cells: usize,
    y_cells: usize,
    collapsed: bool,
}

impl<'a, Data> Wave<'a, Data>
where
    Data: Hash + Clone + Eq,
{
    pub fn new(
        tiles: &'a Vec<Tile<'a, Data>>,
        x_cells: usize,
        y_cells: usize,
        adjacency_map: &PatternAdjacencyMap<'a, Data>,
    ) -> Self {
        let cells: Vec<_> = (0..(x_cells * y_cells))
            .map(|_i| Cell::new(tiles.iter().collect(), adjacency_map.clone()))
            .collect();

        Self {
            cells,
            x_cells,
            y_cells,
            collapsed: false,
        }
    }

    fn get_highest_entropy(&self) -> Vec<usize> {
        let mut cells: Vec<_> = self.cells.iter().enumerate().collect();
        cells.sort_by(|a, b| b.1.entropy().partial_cmp(&a.1.entropy()).unwrap());

        cells.iter().map(|(i, _)| i).cloned().collect()
    }

    fn get_neighbors(&self, index: usize) -> [usize; 4] {
        let row = index / self.x_cells;
        let col = index % self.x_cells;

        let north = ((row + self.y_cells - 1) % self.y_cells) * self.x_cells + col;
        let east = (row * self.x_cells) + ((col + 1) % self.x_cells);
        let south = ((row + 1) % self.y_cells) * self.x_cells + col;
        let west = (row * self.x_cells) + ((col + self.x_cells - 1) % self.x_cells);

        [north, east, south, west]
    }

    pub fn collapse(&mut self) -> bool {
        let mut rng = rand::thread_rng();

        let highest_entropy = self.get_highest_entropy();
        let entropy_index = rng.gen_range(0..highest_entropy.len());

        let index = highest_entropy[entropy_index];
        let cell = &mut self.cells[index];

        let mut removed_tiles: VecDeque<_> = cell
            .collapse()
            .into_iter()
            .map(|tile| RemovedTile {
                cell_index: index,
                tile,
            })
            .collect();

        while let Some(removed) = removed_tiles.pop_front() {
            for (focus_location, focus_index) in
                Location::iter().zip(self.get_neighbors(removed.cell_index))
            {
                dbg!(focus_index);
                let focus = self.cells.get_mut(focus_index).unwrap();

                focus.removed_neighbor_tile(focus_location.opposite(), removed.tile);
            }
        }

        true
    }
}
