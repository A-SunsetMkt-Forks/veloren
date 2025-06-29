pub mod cell;
pub mod mat_cell;
use cell::CellAttr;
pub use mat_cell::Material;

// Reexport
pub use self::{
    cell::{Cell, CellData},
    mat_cell::MatCell,
};

use crate::{
    terrain::{Block, BlockKind, SpriteKind},
    vol::{FilledVox, IntoFullPosIterator, IntoFullVolIterator, ReadVol, SizedVol, WriteVol},
    volumes::dyna::Dyna,
};
use dot_vox::DotVoxData;
use vek::*;

pub type TerrainSegment = Dyna<Block, ()>;

impl From<Segment> for TerrainSegment {
    fn from(value: Segment) -> Self {
        TerrainSegment::from_fn(value.sz, (), |pos| match value.get(pos) {
            Err(_) | Ok(Cell::Empty) => Block::air(SpriteKind::Empty),
            Ok(cell) => {
                if cell.attr().is_hollow() {
                    Block::air(SpriteKind::Empty)
                } else if cell.attr().is_glowy() {
                    Block::new(BlockKind::GlowingRock, cell.get_color().unwrap())
                } else {
                    Block::new(BlockKind::Misc, cell.get_color().unwrap())
                }
            },
        })
    }
}

/// A type representing a volume that may be part of an animated figure.
///
/// Figures are used to represent things like characters, NPCs, mobs, etc.
pub type Segment = Dyna<Cell, ()>;

impl Segment {
    /// Take a list of voxel data, offsets, and x-mirror flags, and assembled
    /// them into a combined segment
    pub fn from_voxes(data: &[(&DotVoxData, Vec3<i32>, bool)]) -> (Self, Vec3<i32>) {
        let mut union = DynaUnionizer::new();
        for (datum, offset, xmirror) in data.iter() {
            union = union.add(Segment::from_vox(datum, *xmirror, 0), *offset);
        }
        union.unify()
    }

    pub fn from_vox_model_index(dot_vox_data: &DotVoxData, model_index: usize) -> Self {
        Self::from_vox(dot_vox_data, false, model_index)
    }

    pub fn from_vox(dot_vox_data: &DotVoxData, flipped: bool, model_index: usize) -> Self {
        if let Some(model) = dot_vox_data.models.get(model_index) {
            let palette = dot_vox_data
                .palette
                .iter()
                .map(|col| Rgb::new(col.r, col.g, col.b))
                .collect::<Vec<_>>();

            let mut segment = Segment::filled(
                Vec3::new(model.size.x, model.size.y, model.size.z),
                Cell::Empty,
                (),
            );

            for voxel in &model.voxels {
                if let Some(&color) = palette.get(voxel.i as usize) {
                    segment
                        .set(
                            Vec3::new(
                                if flipped {
                                    model.size.x as u8 - 1 - voxel.x
                                } else {
                                    voxel.x
                                },
                                voxel.y,
                                voxel.z,
                            )
                            .map(i32::from),
                            Cell::new(color, CellAttr::from_index(voxel.i)),
                        )
                        .unwrap();
                };
            }

            segment
        } else {
            Segment::filled(Vec3::zero(), Cell::Empty, ())
        }
    }

    /// Transform cells
    #[must_use]
    pub fn map(mut self, transform: impl Fn(Cell) -> Option<Cell>) -> Self {
        for pos in self.full_pos_iter() {
            if let Some(new) = transform(*self.get(pos).unwrap()) {
                self.set(pos, new).unwrap();
            }
        }

        self
    }

    /// Transform cell colors
    #[must_use]
    pub fn map_rgb(self, transform: impl Fn(Rgb<u8>) -> Rgb<u8>) -> Self {
        self.map(|cell| {
            cell.get_color()
                .map(|rgb| Cell::new(transform(rgb), cell.attr()))
        })
    }
}

// TODO: move
/// A `Dyna` builder that combines Dynas
pub struct DynaUnionizer<V: FilledVox>(Vec<(Dyna<V, ()>, Vec3<i32>)>);

impl<V: FilledVox + Copy> DynaUnionizer<V> {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self { DynaUnionizer(Vec::new()) }

    #[must_use]
    pub fn add(mut self, dyna: Dyna<V, ()>, offset: Vec3<i32>) -> Self {
        self.0.push((dyna, offset));
        self
    }

    #[must_use]
    pub fn maybe_add(self, maybe: Option<(Dyna<V, ()>, Vec3<i32>)>) -> Self {
        match maybe {
            Some((dyna, offset)) => self.add(dyna, offset),
            None => self,
        }
    }

    pub fn unify(self) -> (Dyna<V, ()>, Vec3<i32>) { self.unify_with(|v, _| v) }

    /// Unify dynamic volumes, with a function that takes (cell, old_cell) and
    /// returns the cell to use.
    pub fn unify_with(self, mut f: impl FnMut(V, V) -> V) -> (Dyna<V, ()>, Vec3<i32>) {
        if self.0.is_empty() {
            return (
                Dyna::filled(Vec3::zero(), V::default_non_filled(), ()),
                Vec3::zero(),
            );
        }

        // Determine size of the new Dyna
        let mut min_point = self.0[0].1;
        let mut max_point = self.0[0].1 + self.0[0].0.size().map(|e| e as i32);
        for (dyna, offset) in self.0.iter().skip(1) {
            let size = dyna.size().map(|e| e as i32);
            min_point = min_point.map2(*offset, std::cmp::min);
            max_point = max_point.map2(offset + size, std::cmp::max);
        }
        let new_size = (max_point - min_point).map(|e| e as u32);
        // Allocate new segment
        let mut combined = Dyna::filled(new_size, V::default_non_filled(), ());
        // Copy segments into combined
        let origin = min_point.map(|e| -e);
        for (dyna, offset) in self.0 {
            for (pos, vox) in dyna.full_vol_iter() {
                if vox.is_filled() {
                    let cell_pos = origin + offset + pos;
                    let old_vox = *combined.get(cell_pos).unwrap();
                    let new_vox = f(*vox, old_vox);
                    combined.set(cell_pos, new_vox).unwrap();
                }
            }
        }

        (combined, origin)
    }
}

pub type MatSegment = Dyna<MatCell, ()>;

impl MatSegment {
    pub fn to_segment(&self, map: impl Fn(Material) -> Rgb<u8>) -> Segment {
        let mut vol = Dyna::filled(self.size(), Cell::Empty, ());
        for (pos, vox) in self.full_vol_iter() {
            let data = match vox {
                MatCell::None => continue,
                MatCell::Mat(mat) => CellData::new(map(*mat), CellAttr::empty()),
                MatCell::Normal(data) => *data,
            };
            vol.set(pos, Cell::Filled(data)).unwrap();
        }
        vol
    }

    /// Transform cells
    #[must_use]
    pub fn map(mut self, transform: impl Fn(MatCell) -> Option<MatCell>) -> Self {
        for pos in self.full_pos_iter() {
            if let Some(new) = transform(*self.get(pos).unwrap()) {
                self.set(pos, new).unwrap();
            }
        }

        self
    }

    /// Transform cell colors
    #[must_use]
    pub fn map_rgb(self, transform: impl Fn(Rgb<u8>) -> Rgb<u8>) -> Self {
        self.map(|cell| match cell {
            MatCell::Normal(data) => Some(MatCell::Normal(CellData {
                col: transform(data.col),
                ..data
            })),
            _ => None,
        })
    }

    pub fn from_vox_model_index(dot_vox_data: &DotVoxData, model_index: usize) -> Self {
        Self::from_vox(dot_vox_data, false, model_index)
    }

    pub fn from_vox(dot_vox_data: &DotVoxData, flipped: bool, model_index: usize) -> Self {
        if let Some(model) = dot_vox_data.models.get(model_index) {
            let palette = dot_vox_data
                .palette
                .iter()
                .map(|col| Rgb::new(col.r, col.g, col.b))
                .collect::<Vec<_>>();

            let mut vol = Dyna::filled(
                Vec3::new(model.size.x, model.size.y, model.size.z),
                MatCell::None,
                (),
            );

            for voxel in &model.voxels {
                let block = match voxel.i {
                    0 => MatCell::Mat(Material::Skin),
                    1 => MatCell::Mat(Material::Hair),
                    2 => MatCell::Mat(Material::EyeDark),
                    3 => MatCell::Mat(Material::EyeLight),
                    4 => MatCell::Mat(Material::SkinDark),
                    5 => MatCell::Mat(Material::SkinLight),
                    7 => MatCell::Mat(Material::EyeWhite),
                    //6 => MatCell::Mat(Material::Clothing),
                    index => {
                        let color = palette
                            .get(index as usize)
                            .copied()
                            .unwrap_or_else(|| Rgb::broadcast(0));
                        MatCell::Normal(CellData::new(color, CellAttr::from_index(index)))
                    },
                };

                vol.set(
                    Vec3::new(
                        if flipped {
                            model.size.x as u8 - 1 - voxel.x
                        } else {
                            voxel.x
                        },
                        voxel.y,
                        voxel.z,
                    )
                    .map(i32::from),
                    block,
                )
                .unwrap();
            }

            vol
        } else {
            Dyna::filled(Vec3::zero(), MatCell::None, ())
        }
    }
}
