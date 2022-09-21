use std::ops::{Index, IndexMut};
use std::vec;

use image::Pixel;
use rand::prelude::SliceRandom;

use crate::slots::{Location, LocationTable};
use crate::tiles::{Tile, TileTable};

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
        // uncomment to ensure underlying contract is upheld by other methods
        //
        // assert_eq!(
        //     self.location_map.iter().all(|(_, c)| *c == 0),
        //     self.location_map[Location::North] == 0
        // );
        self.location_map[Location::North] == 0
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
    sum_weights: f64,
    sum_weight_log_weight: f64,
}

impl<'a, Data> Cell<'a, Data> {
    pub(crate) fn new(
        ways_to_become_tile: TileTable<WaysToBecomeTile>,
        tiles: TileTable<Option<&'a Tile<Data>>>,
    ) -> Self {
        let probs = tiles.iter().filter_map(|o| o.map(|t| t.probability));
        let sum_weights: f64 = probs.clone().sum();
        let sum_weight_log_weight: f64 = probs.map(|p| p * p.log(2.0)).sum();

        Self {
            num_remaining_tiles: tiles.len(),
            ways_to_become_tile,
            tiles,
            sum_weights,
            sum_weight_log_weight,
        }
    }

    pub fn invalid(&self) -> bool {
        self.num_remaining_tiles == 0
    }

    pub fn collapsed(&self) -> bool {
        // uncomment if you need to verify that the assumed contract (updating the underlying datastructures)
        // is upheld by other methods
        //
        // assert_eq!(
        //     self.tiles
        //         .iter()
        //         .cloned()
        //         .filter_map(|o| o)
        //         .collect::<Box<_>>()
        //         .len()
        //         == 1,
        //     self.num_remaining_tiles == 1
        // );

        // assert_eq!(
        //     self.ways_to_become_tile
        //         .iter()
        //         .filter(|a| !a.is_zero())
        //         .collect::<Box<_>>()
        //         .len()
        //         == 1,
        //     self.num_remaining_tiles == 1,
        // );
        self.num_remaining_tiles == 1
    }

    pub fn uncollapsed(&self) -> bool {
        self.num_remaining_tiles >= 2
    }

    pub fn entropy(&self) -> f64 {
        self.sum_weights.log(2.0) - (self.sum_weight_log_weight / self.sum_weights)
    }

    pub fn choosen_tile(&self) -> Option<&Tile<Data>> {
        if self.num_remaining_tiles == 1 {
            self.tiles.iter().cloned().find_map(|o| o)
        } else {
            None
        }
    }

    pub(crate) fn to_image<T: Pixel>(
        &self,
        blend_cell: fn(data: &Box<[&Box<[Data]>]>) -> Box<[T]>,
    ) -> Box<[T]> {
        let remaining_tiles = self
            .tiles
            .iter()
            .filter_map(|option| option.map(|tile| tile.data()))
            .collect();

        blend_cell(&remaining_tiles)
    }

    pub(crate) fn collapse<Rng: rand::Rng>(&mut self, rng: &mut Rng) -> Box<[&'a Tile<Data>]> {
        let remaining_tiles: Box<_> = self
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
        let mut removed = TileTable(vec![None; self.tiles.len()].into_boxed_slice());
        removed[choosen] = Some(choosen);

        std::mem::swap(&mut self.tiles, &mut removed);

        self.num_remaining_tiles = 1;

        let removed: Box<_> = removed.iter().cloned().filter_map(|o| o).collect();
        removed.iter().cloned().for_each(|t| {
            self.ways_to_become_tile[t].clear();
            self.update_entropy_constants(t);
        });

        removed
    }

    pub(crate) fn removed_neighbor_tile(
        &mut self,
        removed: &'a Tile<Data>,
        removed_location: Location,
    ) -> Option<&'a Tile<Data>> {
        let temp = match self.ways_to_become_tile[removed].decrement(removed_location) {
            DecrementWaysToBecomeTileResult::AlreadyZero => None,
            DecrementWaysToBecomeTileResult::NotZero => None,
            DecrementWaysToBecomeTileResult::Zero => self.remove_tile(removed),
        };

        temp
    }

    fn remove_tile(&mut self, removed: &'a Tile<Data>) -> Option<&'a Tile<Data>> {
        let mut out = None;

        std::mem::swap(&mut self.tiles[removed], &mut out);
        self.num_remaining_tiles -= 1;
        self.update_entropy_constants(removed);

        out
    }

    fn update_entropy_constants(&mut self, tile: &Tile<Data>) {
        self.sum_weights -= tile.probability;
        self.sum_weight_log_weight -= tile.probability * tile.probability.log(2.0);
    }
}
