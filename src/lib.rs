use std::collections::VecDeque;
use std::error::Error;

use crate::cells::{Cell, WaysToBecomeTile};
use crate::slots::Location;
use crate::tiles::{Tile, TileTable};

use enum_map::EnumMap;
use rand::prelude::*;
use strum::{Display, IntoEnumIterator};
use tiles::RemovedTile;

pub mod cells;
pub mod patterns;
mod slots;
pub mod tiles;

#[derive(Debug)]
pub struct Wave<'a, Data> {
    cells: Vec<Cell<'a, Data>>,
    x_cells: usize,
    y_cells: usize,
    size: usize,
}

impl<'a, Data: PartialEq> Wave<'a, Data> {
    pub fn new(tiles: &'a Vec<Tile<Data>>, x_cells: usize, y_cells: usize, size: usize) -> Self {
        let mut ways_to_become_tile: TileTable<WaysToBecomeTile> =
            TileTable(vec![WaysToBecomeTile::default(); tiles.len()]);

        for tile in tiles.iter() {
            for neighbor in tiles.iter() {
                for location in Location::iter() {
                    if tile.is_compatible(neighbor, location) {
                        ways_to_become_tile[tile][location] += 1;
                    }
                }
            }
        }

        let tile_table = TileTable(tiles.iter().map(|t| Some(t)).collect());

        let cells: Vec<_> = (0..(x_cells * y_cells))
            .map(|_i| Cell::new(ways_to_become_tile.clone(), tile_table.clone()))
            .collect();

        Self {
            cells,
            x_cells,
            y_cells,
            size,
        }
    }

    fn get_highest_entropy(&self) -> Vec<usize> {
        let mut cells: Vec<_> = self.cells.iter().enumerate().collect();
        cells.sort_by(|a, b| b.1.entropy().partial_cmp(&a.1.entropy()).unwrap());

        cells.iter().map(|(i, _)| i).cloned().collect()
    }

    fn get_neighbors(&self, index: usize) -> EnumMap<Location, usize> {
        let row = index / self.x_cells;
        let col = index % self.x_cells;

        let north = ((row + self.y_cells - 1) % self.y_cells) * self.x_cells + col;
        let east = (row * self.x_cells) + ((col + 1) % self.x_cells);
        let south = ((row + 1) % self.y_cells) * self.x_cells + col;
        let west = (row * self.x_cells) + ((col + self.x_cells - 1) % self.x_cells);

        let mut out = EnumMap::default();
        out[Location::North] = north;
        out[Location::East] = east;
        out[Location::South] = south;
        out[Location::West] = west;

        out
    }

    pub fn collapse(&mut self) -> Result<bool, WaveCollapseError> {
        let mut rng = rand::thread_rng();

        let highest_entropy = self.get_highest_entropy();
        let entropy_index = rng.gen_range(0..highest_entropy.len());

        let index = highest_entropy[entropy_index];
        let cell = &mut self.cells[index];

        let mut removed_tiles: VecDeque<_> = cell
            .collapse(&mut rng)
            .into_iter()
            .map(|tile| RemovedTile {
                cell_index: index,
                tile,
            })
            .collect();

        while let Some(removed) = removed_tiles.pop_front() {
            for (focus_location, focus_index) in self.get_neighbors(removed.cell_index) {
                let focus = &mut self.cells[focus_index];

                if let Some(no_longer_valid) =
                    focus.removed_neighbor_tile(&removed.tile, focus_location.opposite())
                {
                    removed_tiles.push_back(RemovedTile {
                        cell_index: focus_index,
                        tile: no_longer_valid,
                    });
                }

                if focus.invalid() {
                    return Err(WaveCollapseError::InvalidCell(focus_index));
                }
            }
        }

        Ok(false)
    }

    pub fn collapsed(&self) -> bool {
        self.cells.iter().all(|c| c.collapsed())
    }

    pub fn to_image(&self) {
        let mut image = image::RgbImage::new(
            (self.size * self.x_cells) as u32,
            (self.size * self.y_cells) as u32,
        );

        for cell in &self.cells {
            for pixel in cell.pixel_data();
        }
    }
}

#[derive(Debug, Display)]
pub enum WaveCollapseError {
    InvalidCell(usize),
}

impl Error for WaveCollapseError {}
