use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::patterns::Pattern;
use crate::slots::Location;

#[derive(Debug)]
pub struct TileId(usize);

impl<T: Into<usize>> From<T> for TileId {
    fn from(i: T) -> Self {
        TileId(i.into())
    }
}

impl Deref for TileId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TileTable<T>(pub(crate) Box<[T]>);

impl<T> Deref for TileTable<T> {
    type Target = Box<[T]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for TileTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Index<&TileId> for TileTable<T> {
    type Output = T;

    fn index(&self, index: &TileId) -> &Self::Output {
        &self.0[index.0]
    }
}

impl<T> IndexMut<&TileId> for TileTable<T> {
    fn index_mut(&mut self, index: &TileId) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

impl<T, Data> Index<&Tile<Data>> for TileTable<T> {
    type Output = T;

    fn index(&self, index: &Tile<Data>) -> &Self::Output {
        &self.0[index.id.0]
    }
}

impl<T, Data> IndexMut<&Tile<Data>> for TileTable<T> {
    fn index_mut(&mut self, index: &Tile<Data>) -> &mut Self::Output {
        &mut self.0[index.id.0]
    }
}

#[derive(Debug)]
pub struct Tile<Data> {
    pattern: Pattern<Data>,
    pub(crate) probability: f64,
    pub(crate) id: TileId,
}

impl<Data> Tile<Data> {
    pub fn new<T: Into<TileId>>(pattern: Pattern<Data>, probability: f64, id: T) -> Self {
        Self {
            pattern,
            probability,
            id: id.into(),
        }
    }

    pub fn data(&self) -> &Box<[Data]> {
        self.pattern.data()
    }
}

impl<Data: PartialEq> Tile<Data> {
    pub fn is_compatible(&self, b: &Tile<Data>, b_location: Location) -> bool {
        self.pattern.is_compatible(&b.pattern, b_location)
    }
}

#[derive(Debug)]
pub(crate) struct RemovedTile<'a, Data> {
    pub(crate) cell_index: usize,
    pub(crate) tile: &'a Tile<Data>,
}
