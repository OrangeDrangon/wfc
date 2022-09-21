use std::collections::VecDeque;
use std::error::Error;

use crate::cells::{Cell, WaysToBecomeTile};
use crate::slots::Location;
use crate::tiles::{Tile, TileTable};

use enum_map::EnumMap;
use image::{Pixel, RgbaImage};
use rand::prelude::*;
use strum::{Display, IntoEnumIterator};
use tiles::RemovedTile;

pub mod cells;
pub mod patterns;
mod slots;
pub mod tiles;

#[derive(Debug)]
pub struct Wave<'a, Data> {
    cells: Box<[Cell<'a, Data>]>,
    x_cells: usize,
    y_cells: usize,
    size: usize,
    num_collapsed: usize,
}

impl<'a, Data: PartialEq> Wave<'a, Data> {
    pub fn new(tiles: &'a Box<[Tile<Data>]>, x_cells: usize, y_cells: usize, size: usize) -> Self {
        let mut ways_to_become_tile: TileTable<WaysToBecomeTile> =
            TileTable(vec![WaysToBecomeTile::default(); tiles.len()].into_boxed_slice());

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

        let cells: Box<_> = (0..(x_cells * y_cells))
            .map(|_i| Cell::new(ways_to_become_tile.clone(), tile_table.clone()))
            .collect();

        Self {
            cells,
            x_cells,
            y_cells,
            size,
            num_collapsed: 0,
        }
    }

    fn get_lowest_entropy_cells(&self) -> Box<[usize]> {
        let mut uncolapsed_cells: Box<_> = self
            .cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                if c.uncollapsed() {
                    Some((i, c.entropy()))
                } else {
                    None
                }
            })
            .collect();

        uncolapsed_cells.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        assert!(uncolapsed_cells.len() > 0);
        let min = uncolapsed_cells[0].1;

        uncolapsed_cells
            .into_iter()
            .filter_map(|(i, c)| if *c == min { Some(*i) } else { None })
            .collect()
    }

    fn get_neighbors(&self, index: usize) -> EnumMap<Location, usize> {
        let row = index / self.x_cells;
        let col = index % self.x_cells;

        let north = {
            let new_row = (row + self.y_cells - 1) % self.y_cells;
            new_row * self.x_cells + col
        };
        let east = {
            let new_col = (col + 1) % self.x_cells;
            row * self.x_cells + new_col
        };
        let south = {
            let new_row = (row + 1) % self.y_cells;
            new_row * self.x_cells + col
        };
        let west = {
            let new_col = (col + self.x_cells - 1) % self.x_cells;
            (row * self.x_cells) + new_col
        };

        let mut out = EnumMap::default();
        out[Location::North] = north;
        out[Location::East] = east;
        out[Location::South] = south;
        out[Location::West] = west;

        out
    }

    pub fn collapse(&mut self) -> Result<bool, WaveCollapseError> {
        if self.collapsed() {
            return Err(WaveCollapseError::AlreadyCollapsed);
        }

        let mut rng = rand::thread_rng();

        let lowest_entropy = self.get_lowest_entropy_cells();
        let entropy_index = rng.gen_range(0..lowest_entropy.len());

        let index = lowest_entropy[entropy_index];
        let cell = &mut self.cells[index];

        let mut removed_tiles: VecDeque<_> = cell
            .collapse(&mut rng)
            .into_iter()
            .map(|tile| RemovedTile {
                cell_index: index,
                tile,
            })
            .collect();

        self.num_collapsed += 1;

        while !self.collapsed() && removed_tiles.len() > 0 {
            let removed = removed_tiles.pop_front().unwrap();

            for (focus_location, focus_index) in self.get_neighbors(removed.cell_index) {
                let focus = &mut self.cells[focus_index];
                let focus_already_collapsed = focus.collapsed();

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
                } else if !focus_already_collapsed && focus.collapsed() {
                    self.num_collapsed += 1;
                }
            }
        }

        Ok(false)
    }

    pub fn collapsed(&self) -> bool {
        // uncomment to verify the underlying contract is upheld where cells are collapsed
        // before the num collapsed is incremented
        //
        // assert_eq!(
        //     self.cells.iter().all(|c| c.collapsed()),
        //     self.num_collapsed == self.cells.len()
        // );
        self.num_collapsed == self.cells.len()
    }

    pub fn to_image<T: Pixel<Subpixel = u8>>(
        &self,
        blend_cell: fn(data: &Box<[&Box<[Data]>]>) -> Box<[T]>,
    ) -> RgbaImage {
        let size_padding = self.size + 2;
        let mut image = image::RgbaImage::new(
            (size_padding * self.x_cells) as u32,
            (size_padding * self.y_cells) as u32,
        );

        for (i, cell) in self.cells.iter().enumerate() {
            let pixels = cell.to_image(blend_cell);

            let row = i / self.y_cells;
            let col = i % self.x_cells;

            let cell_x = col * size_padding + 1;
            let cell_y = row * size_padding + 1;

            for (j, pixel) in pixels.into_iter().enumerate() {
                let local_x = j % self.size;
                let local_y = j / self.size;

                let x = (local_x + cell_x) as u32 % image.width();
                let y = (local_y + cell_y) as u32 % image.height();

                image.put_pixel(x, y, pixel.to_rgba());
            }
        }

        image
    }
}

#[derive(Debug, Display)]
pub enum WaveCollapseError {
    InvalidCell(usize),
    AlreadyCollapsed,
}

impl Error for WaveCollapseError {}
