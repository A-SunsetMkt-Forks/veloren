use super::*;
use crate::{
    Land,
    site::{gen::wall_staircase, util::sprites::PainterSpriteExt},
    util::{CARDINALS, NEIGHBORS, RandomField, Sampler},
};
use common::terrain::{BlockKind, SpriteKind};
use rand::prelude::*;
use std::sync::Arc;
use strum::IntoEnumIterator;
use vek::*;

/// Represents house data generated by the `generate()` method
pub struct CoastalHouse {
    /// Tile position of the door tile
    pub door_tile: Vec2<i32>,
    /// Axis aligned bounding region for the house
    bounds: Aabr<i32>,
    /// Approximate altitude of the door tile
    pub(crate) alt: i32,
}

impl CoastalHouse {
    pub fn generate(
        land: &Land,
        _rng: &mut impl Rng,
        site: &Site,
        door_tile: Vec2<i32>,
        door_dir: Vec2<i32>,
        tile_aabr: Aabr<i32>,
        alt: Option<i32>,
    ) -> Self {
        let door_tile_pos = site.tile_center_wpos(door_tile);
        let bounds = Aabr {
            min: site.tile_wpos(tile_aabr.min),
            max: site.tile_wpos(tile_aabr.max),
        };
        Self {
            door_tile: door_tile_pos,
            bounds,
            alt: alt.unwrap_or_else(|| {
                land.get_alt_approx(site.tile_center_wpos(door_tile + door_dir)) as i32
            }) + 2,
        }
    }
}

impl Structure for CoastalHouse {
    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"render_coastalhouse\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "render_coastalhouse"))]
    fn render_inner(&self, _site: &Site, _land: &Land, painter: &Painter) {
        let base = self.alt + 1;
        let center = self.bounds.center();
        let white = Fill::Sampling(Arc::new(|center| {
            Some(match (RandomField::new(0).get(center)) % 37 {
                0..=8 => Block::new(BlockKind::Rock, Rgb::new(251, 251, 227)),
                9..=17 => Block::new(BlockKind::Rock, Rgb::new(245, 245, 229)),
                18..=26 => Block::new(BlockKind::Rock, Rgb::new(250, 243, 221)),
                27..=35 => Block::new(BlockKind::Rock, Rgb::new(240, 240, 230)),
                _ => Block::new(BlockKind::Rock, Rgb::new(255, 244, 193)),
            })
        }));
        let blue_broken = Fill::Sampling(Arc::new(|center| {
            Some(match (RandomField::new(0).get(center)) % 20 {
                0 => Block::new(BlockKind::Rock, Rgb::new(30, 187, 235)),
                _ => Block::new(BlockKind::Rock, Rgb::new(11, 146, 187)),
            })
        }));
        let length = (14 + RandomField::new(0).get(center.with_z(base)) % 3) as i32;
        let width = (12 + RandomField::new(0).get((center - 1).with_z(base)) % 3) as i32;
        let height = (12 + RandomField::new(0).get((center + 1).with_z(base)) % 4) as i32;
        let storeys = (1 + RandomField::new(0).get(center.with_z(base)) % 2) as i32;

        // fence, blue gates
        painter
            .aabb(Aabb {
                min: Vec2::new(center.x - length - 6, center.y - width - 6).with_z(base - 2),
                max: Vec2::new(center.x + length + 7, center.y + width + 7).with_z(base - 1),
            })
            .fill(blue_broken.clone());

        for dir in CARDINALS {
            let frame_pos = Vec2::new(
                center.x + dir.x * (length + 5),
                center.y + dir.y * (width + 5),
            );
            painter
                .line(center.with_z(base - 1), frame_pos.with_z(base - 1), 3.0)
                .fill(blue_broken.clone());
        }
        // foundation
        painter
            .aabb(Aabb {
                min: Vec2::new(center.x - length - 6, center.y - width - 6).with_z(base - height),
                max: Vec2::new(center.x + length + 7, center.y + width + 7).with_z(base - 2),
            })
            .fill(white.clone());
        for f in 0..8 {
            painter
                .aabb(Aabb {
                    min: Vec2::new(center.x - length - 7 - f, center.y - width - 7 - f)
                        .with_z(base - 3 - f),
                    max: Vec2::new(center.x + length + 8 + f, center.y + width + 8 + f)
                        .with_z(base - 2 - f),
                })
                .fill(white.clone());
        }
        // clear yard
        painter
            .aabb(Aabb {
                min: Vec2::new(center.x - length - 5, center.y - width - 5).with_z(base - 2),
                max: Vec2::new(center.x + length + 6, center.y + width + 6).with_z(base + height),
            })
            .clear();
        // clear entries
        for dir in CARDINALS {
            let clear_pos = Vec2::new(
                center.x + dir.x * (length + 7),
                center.y + dir.y * (width + 7),
            );
            painter
                .line(center.with_z(base - 1), clear_pos.with_z(base - 1), 2.0)
                .clear();
        }
        for s in 0..storeys {
            // roof terrace
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length - 3 + (2 * s),
                        center.y - width - 3 + (2 * s),
                    )
                    .with_z(base - 3 + height + (s * height)),
                    max: Vec2::new(
                        center.x + length + 2 - (2 * s),
                        center.y + width + 2 - (2 * s),
                    )
                    .with_z(base - 2 + height + (s * height)),
                })
                .fill(white.clone());
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length - 3 + (2 * s),
                        center.y - width - 3 + (2 * s),
                    )
                    .with_z(base - 2 + height + (s * height)),
                    max: Vec2::new(
                        center.x + length + 2 - (2 * s),
                        center.y + width + 2 - (2 * s),
                    )
                    .with_z(base - 1 + height + (s * height)),
                })
                .fill(blue_broken.clone());
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length - 2 + (2 * s),
                        center.y - width - 2 + (2 * s),
                    )
                    .with_z(base - 2 + height + (s * height)),
                    max: Vec2::new(
                        center.x + length + 1 - (2 * s),
                        center.y + width + 1 - (2 * s),
                    )
                    .with_z(base - 1 + height + (s * height)),
                })
                .clear();
            // room
            painter
                .aabb(Aabb {
                    min: Vec2::new(center.x - length + (2 * s), center.y - width + (2 * s))
                        .with_z(base - 2 + (s * height)),
                    max: Vec2::new(center.x + length - (2 * s), center.y + width - (2 * s))
                        .with_z(base - 1 + (s * height)),
                })
                .fill(blue_broken.clone());
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length + 1 + (2 * s),
                        center.y - width + 1 + (2 * s),
                    )
                    .with_z(base - 2 + (s * height)),
                    max: Vec2::new(
                        center.x + length - 1 - (2 * s),
                        center.y + width - 1 - (2 * s),
                    )
                    .with_z(base - 1 + height - 1 + (s * height)),
                })
                .fill(white.clone());

            // entries
            painter
                .line(
                    Vec2::new(center.x, center.y + 1 - width + (2 * s))
                        .with_z(base - 1 + (s * height)),
                    Vec2::new(center.x, center.y - 2 + width - (2 * s))
                        .with_z(base - 1 + (s * height)),
                    3.0,
                )
                .fill(blue_broken.clone());
            painter
                .line(
                    Vec2::new(center.x, center.y - width + (2 * s)).with_z(base - 1 + (s * height)),
                    Vec2::new(center.x, center.y + width - (2 * s)).with_z(base - 1 + (s * height)),
                    2.0,
                )
                .clear();
            painter
                .line(
                    Vec2::new(center.x + 1 - length + (2 * s), center.y)
                        .with_z(base - 1 + (s * height)),
                    Vec2::new(center.x - 2 + length - (2 * s), center.y)
                        .with_z(base - 1 + (s * height)),
                    3.0,
                )
                .fill(blue_broken.clone());
            painter
                .line(
                    Vec2::new(center.x - length + (2 * s), center.y)
                        .with_z(base - 1 + (s * height)),
                    Vec2::new(center.x + length - (2 * s), center.y)
                        .with_z(base - 1 + (s * height)),
                    2.0,
                )
                .clear();
            // windows length
            painter
                .line(
                    Vec2::new(center.x - (length / 3), center.y + 1 - width + (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    Vec2::new(center.x - (length / 3), center.y - 2 + width - (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    3.0,
                )
                .fill(blue_broken.clone());
            painter
                .line(
                    Vec2::new(center.x - (length / 3), center.y - width - 2 + (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    Vec2::new(center.x - (length / 3), center.y + width + 2 - (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    2.0,
                )
                .clear();

            painter
                .line(
                    Vec2::new(center.x + (length / 3), center.y + 1 - width + (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    Vec2::new(center.x + (length / 3), center.y - 2 + width - (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    3.0,
                )
                .fill(blue_broken.clone());
            painter
                .line(
                    Vec2::new(center.x + (length / 3), center.y - width - 2 + (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    Vec2::new(center.x + (length / 3), center.y + width + 2 - (2 * s))
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    2.0,
                )
                .clear();

            // windows width
            painter
                .line(
                    Vec2::new(center.x + 1 - length + (2 * s), center.y)
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    Vec2::new(center.x - 2 + length - (2 * s), center.y)
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    3.0,
                )
                .fill(blue_broken.clone());
            painter
                .line(
                    Vec2::new(center.x - length - 2 + (2 * s), center.y)
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    Vec2::new(center.x + length + 2 - (2 * s), center.y)
                        .with_z(base - 1 + (s * height) + (height / 2)),
                    2.0,
                )
                .clear();

            // clear room
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length + 2 + (2 * s),
                        center.y - width + 2 + (2 * s),
                    )
                    .with_z(base - 2 + (s * height)),
                    max: Vec2::new(
                        center.x + length - 2 - (2 * s),
                        center.y + width - 2 - (2 * s),
                    )
                    .with_z(base - 2 + height - 1 + (s * height)),
                })
                .clear();

            // room floors
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length + 5 + (2 * s),
                        center.y - width + 5 + (2 * s),
                    )
                    .with_z(base - 3 + (s * height)),
                    max: Vec2::new(
                        center.x + length - 5 - (2 * s),
                        center.y + width - 5 - (2 * s),
                    )
                    .with_z(base - 2 + (s * height)),
                })
                .fill(blue_broken.clone());
            painter
                .aabb(Aabb {
                    min: Vec2::new(
                        center.x - length + 6 + (2 * s),
                        center.y - width + 6 + (2 * s),
                    )
                    .with_z(base - 3 + (s * height)),
                    max: Vec2::new(
                        center.x + length - 6 - (2 * s),
                        center.y + width - 6 - (2 * s),
                    )
                    .with_z(base - 2 + (s * height)),
                })
                .fill(white.clone());
            // furniture
            let mut sprites = vec![
                SpriteKind::Bowl,
                SpriteKind::VialEmpty,
                SpriteKind::FountainArabic,
                SpriteKind::Crate,
                SpriteKind::Lantern,
            ];
            'outer: for dir in NEIGHBORS {
                let furniture_pos = Vec2::new(
                    center.x + dir.x * ((length / 2) + 1),
                    center.y + dir.y * ((width / 2) + 1),
                );
                if sprites.is_empty() {
                    break 'outer;
                }
                let sprite = sprites.swap_remove(
                    RandomField::new(0).get(furniture_pos.with_z(base)) as usize % sprites.len(),
                );
                painter.owned_resource_sprite(
                    furniture_pos.with_z(base - 2 + (s * height)),
                    sprite,
                    0,
                );
            }

            // clear floor center if stairs
            if storeys > 1 {
                painter
                    .cylinder(Aabb {
                        min: (center - 6).with_z(base - 2 + (s * height)),
                        max: (center + 6).with_z(base + (s * height)),
                    })
                    .clear();
            };

            // draws a random index based of base and currently storey
            let random_index_1 = (RandomField::new(0).get(center.with_z(base + s)) % 4) as usize;
            let random_index_2 = 3 - random_index_1;
            // add beds and tables at random corners
            for (d, dir) in Dir::iter().enumerate() {
                let diagonal = dir.diagonal();
                let bed_pos = center + diagonal * ((length / 2) - 2);
                let table_pos = Vec2::new(
                    center.x + diagonal.x * ((length / 2) - 1),
                    center.y + diagonal.y * ((width / 2) - 2),
                );
                let alt = base - 2 + (s * height);
                if d == random_index_1 {
                    painter.bed_coastal(bed_pos.with_z(alt), dir);
                } else if d == random_index_2 {
                    painter.rotated_sprite(table_pos.with_z(alt), SpriteKind::TableCoastalLarge, 2);
                    painter.sprite(table_pos.with_z(alt + 1), SpriteKind::JugAndCupsCoastal);

                    for dir in Dir::iter() {
                        let vec = dir.to_vec2();
                        let bench_pos = Vec2::new(table_pos.x + vec.x * 2, table_pos.y + vec.y);
                        painter.rotated_sprite(
                            bench_pos.with_z(alt),
                            SpriteKind::BenchCoastal,
                            dir.opposite().sprite_ori(),
                        );
                    }
                }
            }

            // wall lamps
            for d in 0..2 {
                let door_lamp_pos = Vec2::new(
                    center.x - length + 2 + (2 * s) + (d * ((2 * (length - (2 * s))) - 5)),
                    center.y,
                )
                .with_z(base + 1 + (s * height));
                painter.rotated_sprite(
                    door_lamp_pos,
                    SpriteKind::WallLampSmall,
                    2 + ((d * 4) as u8),
                );

                let lamp_pos = Vec2::new(
                    center.x,
                    center.y - width + 2 + (2 * s) + (d * ((2 * (width - (2 * s))) - 5)),
                )
                .with_z(base + 1 + (s * height));
                painter.rotated_sprite(lamp_pos, SpriteKind::WallLampSmall, 4 - ((d * 4) as u8));
            }
            for d in 0..2 {
                let door_lamp_pos = Vec2::new(
                    center.x - length - 1 + (2 * s) + (d * ((2 * (length - (2 * s))) + 1)),
                    center.y,
                )
                .with_z(base + 1 + (s * height));
                painter.rotated_sprite(
                    door_lamp_pos,
                    SpriteKind::WallLampSmall,
                    6 + ((d * 4) as u8),
                );

                let lamp_pos = Vec2::new(
                    center.x,
                    center.y - width - 1 + (2 * s) + (d * ((2 * (width - (2 * s))) + 1)),
                )
                .with_z(base + 1 + (s * height));
                painter.rotated_sprite(lamp_pos, SpriteKind::WallLampSmall, 8 - ((d * 4) as u8));
            }
        }
        let top_limit = painter.aabb(Aabb {
            min: Vec2::new(center.x - length, center.y - width)
                .with_z(base + (storeys * height) - 2),
            max: Vec2::new(center.x + length, center.y + width)
                .with_z(base - 2 + (storeys * height) + (height / 2)),
        });
        painter
            .superquadric(
                Aabb {
                    min: Vec2::new(center.x - length - 1, center.y - width - 1)
                        .with_z(base + (storeys * height) - (height / 2)),
                    max: Vec2::new(center.x + length, center.y + width)
                        .with_z(base - 2 + (storeys * height) + (height / 2)),
                },
                1.5,
            )
            .intersect(top_limit)
            .fill(white.clone());
        if storeys > 1 {
            // stairway1 stairs
            let stair_radius1 = 3.0;
            let stairs_clear1 = painter.cylinder(Aabb {
                min: (center - 1 - stair_radius1 as i32).with_z(base - 2),
                max: (center + 2 + stair_radius1 as i32)
                    .with_z(base + ((storeys - 1) * height) - 2),
            });
            let stairs_clear2 = painter.cylinder(Aabb {
                min: (center - 2 - stair_radius1 as i32).with_z(base - 2),
                max: (center + 3 + stair_radius1 as i32)
                    .with_z(base + ((storeys - 1) * height) - 2),
            });
            stairs_clear1.clear();
            painter
                .cylinder(Aabb {
                    min: (center - 1).with_z(base - 2),
                    max: (center + 2).with_z(base + ((storeys - 1) * height) - 2),
                })
                .fill(white.clone());

            stairs_clear2
                .sample(wall_staircase(
                    center.with_z(base + ((storeys - 1) * height) - 2),
                    stair_radius1,
                    (height / 2) as f32,
                ))
                .fill(white);
        }
    }
}
