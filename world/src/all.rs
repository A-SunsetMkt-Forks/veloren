use crate::{
    IndexRef,
    util::{math::close, sampler::Sampler},
};
use common::{match_some, terrain::structure::StructureBlock};
use std::ops::Range;
use strum::EnumIter;
use vek::*;

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum ForestKind {
    Palm,
    Acacia,
    Baobab,
    Oak,
    Chestnut,
    Cedar,
    Pine,
    Redwood,
    Birch,
    Mangrove,
    Giant,
    Swamp,
    Frostpine,
    Dead,
    Mapletree,
    Cherry,
    AutumnTree,
}

pub struct Environment {
    pub humid: f32,
    pub temp: f32,
    pub near_water: f32,
}

impl ForestKind {
    pub fn humid_range(&self) -> Range<f32> {
        match self {
            ForestKind::Palm => 0.25..1.4,
            ForestKind::Acacia => 0.05..0.55,
            ForestKind::Baobab => 0.2..0.6,
            ForestKind::Oak => 0.35..1.5,
            ForestKind::Chestnut => 0.35..1.5,
            ForestKind::Cedar => 0.275..1.45,
            ForestKind::Pine => 0.2..1.4,
            ForestKind::Redwood => 0.6..1.0,
            ForestKind::Frostpine => 0.2..1.4,
            ForestKind::Birch => 0.0..0.6,
            ForestKind::Mangrove => 0.5..1.3,
            ForestKind::Swamp => 0.5..1.1,
            ForestKind::Dead => 0.0..1.5,
            ForestKind::Mapletree => 0.65..1.25,
            ForestKind::Cherry => 0.45..0.75,
            ForestKind::AutumnTree => 0.25..0.65,
            _ => 0.0..0.0,
        }
    }

    pub fn temp_range(&self) -> Range<f32> {
        match self {
            ForestKind::Palm => 0.4..1.6,
            ForestKind::Acacia => 0.3..1.6,
            ForestKind::Baobab => 0.4..0.9,
            ForestKind::Oak => -0.35..0.45,
            ForestKind::Chestnut => -0.35..0.45,
            ForestKind::Cedar => -0.65..0.15,
            ForestKind::Pine => -0.85..-0.2,
            ForestKind::Redwood => -0.5..-0.3,
            ForestKind::Frostpine => -1.8..-0.8,
            ForestKind::Birch => -0.7..0.25,
            ForestKind::Mangrove => 0.35..1.6,
            ForestKind::Swamp => -0.6..0.8,
            ForestKind::Dead => -1.5..1.0,
            ForestKind::Mapletree => -0.15..0.25,
            ForestKind::Cherry => -0.10..0.15,
            ForestKind::AutumnTree => -0.45..0.05,
            _ => 0.0..0.0,
        }
    }

    pub fn near_water_range(&self) -> Option<Range<f32>> {
        match_some!(self,
            ForestKind::Palm => 0.35..1.8,
            ForestKind::Swamp => 0.5..1.8,
        )
    }

    /// The relative rate at which this tree appears under ideal conditions
    pub fn ideal_proclivity(&self) -> f32 {
        match self {
            ForestKind::Palm => 0.4,
            ForestKind::Acacia => 0.6,
            ForestKind::Baobab => 0.2,
            ForestKind::Oak => 1.0,
            ForestKind::Chestnut => 0.3,
            ForestKind::Cedar => 0.3,
            ForestKind::Pine => 1.0,
            ForestKind::Redwood => 2.5,
            ForestKind::Frostpine => 1.0,
            ForestKind::Birch => 0.65,
            ForestKind::Mangrove => 2.0,
            ForestKind::Swamp => 1.0,
            ForestKind::Dead => 0.01,
            ForestKind::Mapletree => 0.65,
            ForestKind::Cherry => 12.0,
            ForestKind::AutumnTree => 125.0,
            _ => 0.0,
        }
    }

    pub fn shrub_density_factor(&self) -> f32 {
        match self {
            ForestKind::Palm => 0.2,
            ForestKind::Acacia => 0.3,
            ForestKind::Baobab => 0.2,
            ForestKind::Oak => 0.4,
            ForestKind::Chestnut => 0.3,
            ForestKind::Cedar => 0.3,
            ForestKind::Pine => 0.5,
            ForestKind::Frostpine => 0.3,
            ForestKind::Birch => 0.65,
            ForestKind::Mangrove => 1.0,
            ForestKind::Swamp => 0.4,
            ForestKind::Mapletree => 0.4,
            ForestKind::Cherry => 0.3,
            ForestKind::AutumnTree => 0.4,
            _ => 1.0,
        }
    }

    pub fn leaf_block(&self) -> StructureBlock {
        match self {
            ForestKind::Palm => StructureBlock::PalmLeavesOuter,
            ForestKind::Acacia => StructureBlock::Acacia,
            ForestKind::Baobab => StructureBlock::Baobab,
            ForestKind::Oak => StructureBlock::TemperateLeaves,
            ForestKind::Chestnut => StructureBlock::Chestnut,
            ForestKind::Cedar => StructureBlock::PineLeaves,
            ForestKind::Pine => StructureBlock::PineLeaves,
            ForestKind::Redwood => StructureBlock::PineLeaves,
            ForestKind::Birch => StructureBlock::TemperateLeaves,
            ForestKind::Mangrove => StructureBlock::Mangrove,
            ForestKind::Giant => StructureBlock::TemperateLeaves,
            ForestKind::Swamp => StructureBlock::TemperateLeaves,
            ForestKind::Frostpine => StructureBlock::FrostpineLeaves,
            ForestKind::Dead => StructureBlock::TemperateLeaves,
            ForestKind::Mapletree => StructureBlock::MapleLeaves,
            ForestKind::Cherry => StructureBlock::CherryLeaves,
            ForestKind::AutumnTree => StructureBlock::AutumnLeaves,
        }
    }

    pub fn proclivity(&self, env: &Environment) -> f32 {
        self.ideal_proclivity()
            * close(env.humid, self.humid_range())
            * close(env.temp, self.temp_range())
            * self.near_water_range().map_or(1.0, |near_water_range| {
                close(env.near_water, near_water_range)
            })
    }
}

pub fn leaf_color(
    index: IndexRef,
    seed: u32,
    lerp: f32,
    sblock: &StructureBlock,
) -> Option<Rgb<u8>> {
    let ranges = sblock
        .elim_case_pure(&index.colors.block.structure_blocks)
        .as_ref()
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    ranges
        .get(crate::util::RandomPerm::new(seed).get(seed) as usize % ranges.len())
        .map(|range| {
            Rgb::<f32>::lerp(
                Rgb::<u8>::from(range.start).map(f32::from),
                Rgb::<u8>::from(range.end).map(f32::from),
                lerp,
            )
            .map(|e| e as u8)
        })
}

/// Not currently used with trees generated by the tree layer, needs to be
/// reworked
pub struct TreeAttr {
    pub pos: Vec2<i32>,
    pub seed: u32,
    pub scale: f32,
    pub forest_kind: ForestKind,
    pub inhabited: bool,
}
