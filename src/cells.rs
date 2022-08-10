use std::ops::{Index, IndexMut};
use std::vec;

use rand::prelude::SliceRandom;

use crate::slots::{Location, LocationTable};
use crate::tiles::{RemovedTile, Tile, TileTable};

enum DecrementWaysToBecomeTileResult {
    AlreadyZero,
    NotZero,
    Zero,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct WaysToBecomeTile {
    location_map: LocationTable<usize>,
}

impl WaysToBecomeTile {
    fn is_zero(&self) -> bool {
        self.location_map[Location::North] == 0usize
    }

    fn clear(&mut self) {
        self.location_map = LocationTable::default();
    }

    fn decrement(&mut self, location: Location) -> DecrementWaysToBecomeTileResult {
        match &mut self.location_map[location] {
            0 => DecrementWaysToBecomeTileResult::AlreadyZero,
            1 => {
                self.clear();
                DecrementWaysToBecomeTileResult::Zero
            }
            _ => {
                self.location_map[location] -= 1;
                DecrementWaysToBecomeTileResult::NotZero
            }
        }
    }
}

impl Index<Location> for WaysToBecomeTile {
    type Output = usize;

    fn index(&self, index: Location) -> &Self::Output {
        &self.location_map[index]
    }
}

impl IndexMut<Location> for WaysToBecomeTile {
    fn index_mut(&mut self, index: Location) -> &mut Self::Output {
        &mut self.location_map[index]
    }
}

#[derive(Debug)]
pub(crate) struct Cell<'a, Data> {
    ways_to_become_tile: TileTable<WaysToBecomeTile>,
    tiles: TileTable<Option<&'a Tile<Data>>>,
    num_remaining_tiles: usize,
}

impl<'a, Data> Cell<'a, Data> {
    pub(crate) fn new(
        ways_to_become_tile: TileTable<WaysToBecomeTile>,
        tiles: TileTable<Option<&'a Tile<Data>>>,
    ) -> Self {
        Self {
            num_remaining_tiles: tiles.len(),
            ways_to_become_tile,
            tiles,
        }
    }

    pub fn invalid(&self) -> bool {
        self.num_remaining_tiles == 0
    }

    pub fn collapsed(&self) -> bool {
        self.num_remaining_tiles == 1
    }

    pub fn uncollapsed(&self) -> bool {
        self.num_remaining_tiles >= 2
    }

    pub fn entropy(&self) -> f64 {
        -self
            .tiles
            .iter()
            .filter_map(|option| option.map(|tile| tile.probability * tile.probability.log(2.0)))
            .fold(0.0, |shannons, shannon| shannons + shannon)
    }

    pub(crate) fn collapse<Rng: rand::Rng>(&mut self, rng: &mut Rng) -> Vec<&'a Tile<Data>> {
        let remaining_tiles: Vec<_> = self
            .tiles
            .iter()
            .cloned()
            .filter_map(|option| option)
            .collect();

        let choosen = remaining_tiles
            .choose_weighted(rng, |t| t.probability)
            .unwrap()
            .clone();

        // in the future consider not allocating so much leveraging remaining tiles
        self.tiles[choosen] = None;
        let mut removed = TileTable(vec![None; self.tiles.len()]);
        removed[choosen] = Some(choosen);

        std::mem::swap(&mut self.tiles, &mut removed);

        self.num_remaining_tiles = 1;

        let removed: Vec<_> = removed.iter().cloned().filter_map(|o| o).collect();
        removed
            .iter()
            .cloned()
            .for_each(|e| self.ways_to_become_tile[e].clear());

        removed
    }

    pub(crate) fn removed_neighbor_tile(
        &mut self,
        removed: &'a Tile<Data>,
        removed_location: Location,
    ) -> Option<&'a Tile<Data>> {
        match self.ways_to_become_tile[removed].decrement(removed_location) {
            DecrementWaysToBecomeTileResult::AlreadyZero => None,
            DecrementWaysToBecomeTileResult::NotZero => None,
            DecrementWaysToBecomeTileResult::Zero => self.remove_tile(removed),
        }
    }

    fn remove_tile(&mut self, removed: &'a Tile<Data>) -> Option<&'a Tile<Data>> {
        let mut out = None;

        std::mem::swap(&mut self.tiles[removed], &mut out);
        self.num_remaining_tiles -= 1;

        out
    }
}
