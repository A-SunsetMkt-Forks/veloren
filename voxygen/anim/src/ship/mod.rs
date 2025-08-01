pub mod idle;

// Reexports
pub use self::idle::IdleAnimation;

use super::{FigureBoneData, Skeleton, vek::*};
use common::comp::{self};
use core::convert::TryFrom;

pub type Body = comp::ship::Body;

skeleton_impls!(struct ShipSkeleton ComputedShipSkeleton {
    + bone0
    + bone1
    + bone2
    + bone3
});

impl Skeleton for ShipSkeleton {
    type Attr = SkeletonAttr;
    type Body = Body;
    type ComputedSkeleton = ComputedShipSkeleton;

    const BONE_COUNT: usize = ComputedShipSkeleton::BONE_COUNT;
    #[cfg(feature = "use-dyn-lib")]
    const COMPUTE_FN: &'static [u8] = b"ship_compute_mats\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "ship_compute_mats"))]
    fn compute_matrices_inner(
        &self,
        base_mat: Mat4<f32>,
        buf: &mut [FigureBoneData; super::MAX_BONE_COUNT],
        _body: Self::Body,
    ) -> Self::ComputedSkeleton {
        // Ships are normal scale
        let scale_mat = Mat4::scaling_3d(1.0);

        let bone0_mat = base_mat * scale_mat * Mat4::<f32>::from(self.bone0);
        let bone1_mat = bone0_mat * Mat4::<f32>::from(self.bone1);
        let bone2_mat = bone0_mat * Mat4::<f32>::from(self.bone2);

        let computed_skeleton = ComputedShipSkeleton {
            bone0: bone0_mat,
            bone1: bone1_mat,
            bone2: bone2_mat,
            bone3: bone0_mat * Mat4::<f32>::from(self.bone3),
        };

        computed_skeleton.set_figure_bone_data(buf);
        computed_skeleton
    }
}

pub struct SkeletonAttr {
    bone0: (f32, f32, f32),
    bone1: (f32, f32, f32),
    bone2: (f32, f32, f32),
    bone3: (f32, f32, f32),
    bone1_ori: f32,
    bone2_ori: f32,
    bone_rotation_rate: f32,
    pub bone1_prop_trail_offset: Option<f32>,
    pub bone2_prop_trail_offset: Option<f32>,
}

impl<'a> TryFrom<&'a comp::Body> for SkeletonAttr {
    type Error = ();

    fn try_from(body: &'a comp::Body) -> Result<Self, Self::Error> {
        match body {
            comp::Body::Ship(body) => Ok(SkeletonAttr::from(body)),
            _ => Err(()),
        }
    }
}

impl Default for SkeletonAttr {
    fn default() -> Self {
        Self {
            bone0: (0.0, 0.0, 0.0),
            bone1: (0.0, 0.0, 0.0),
            bone2: (0.0, 0.0, 0.0),
            bone3: (0.0, 0.0, 0.0),
            bone1_ori: 0.0,
            bone2_ori: 0.0,
            bone_rotation_rate: 0.0,
            bone1_prop_trail_offset: None,
            bone2_prop_trail_offset: None,
        }
    }
}

impl<'a> From<&'a Body> for SkeletonAttr {
    fn from(body: &'a Body) -> Self {
        use comp::ship::Body::*;
        Self {
            bone0: match body {
                DefaultAirship => (0.0, 0.0, 0.0),
                AirBalloon => (0.0, 0.0, 0.0),
                SailBoat => (0.0, 0.0, 0.0),
                Galleon => (0.0, 0.0, 0.0),
                Skiff => (0.0, 0.0, 0.0),
                Submarine => (0.0, 0.0, 0.0),
                Carriage => (0.0, 0.0, 0.0),
                Cart => (0.0, 0.0, 0.0),
                Volume => (0.0, 0.0, 0.0),
            },
            bone1: match body {
                DefaultAirship => (-13.0, -25.0, 10.0),
                AirBalloon => (0.0, 0.0, 0.0),
                SailBoat => (0.0, 0.0, 0.0),
                Galleon => (0.0, 0.0, 0.0),
                Skiff => (0.0, 0.0, 0.0),
                Submarine => (0.0, -15.0, 3.5),
                Carriage => (0.0, 3.0, 2.0),
                Cart => (0.0, 1.0, 1.0),
                Volume => (0.0, 0.0, 0.0),
            },
            bone2: match body {
                DefaultAirship => (13.0, -25.0, 10.0),
                AirBalloon => (0.0, 0.0, 0.0),
                SailBoat => (0.0, 0.0, 0.0),
                Galleon => (0.0, 0.0, 0.0),
                Skiff => (0.0, 0.0, 0.0),
                Submarine => (0.0, 0.0, 0.0),
                Carriage => (0.0, -3.0, 2.0),
                Cart => (0.0, -2.5, 1.0),
                Volume => (0.0, 0.0, 0.0),
            },
            bone3: match body {
                DefaultAirship => (0.0, -27.5, 8.5),
                AirBalloon => (0.0, -9.0, 8.0),
                SailBoat => (0.0, 0.0, 0.0),
                Galleon => (0.0, 0.0, 0.0),
                Skiff => (0.0, 0.0, 0.0),
                Submarine => (0.0, -18.0, 3.5),
                Carriage => (0.0, 0.0, 0.0),
                Cart => (0.0, 0.0, 0.0),
                Volume => (0.0, 0.0, 0.0),
            },
            bone1_ori: match body {
                Carriage | Cart => std::f32::consts::PI * 0.5,
                _ => 0.0,
            },
            bone2_ori: match body {
                Carriage | Cart => std::f32::consts::PI * -0.5,
                _ => 0.0,
            },
            bone_rotation_rate: match body {
                Carriage => 0.25,
                Cart => 0.4,
                _ => 0.8,
            },
            bone1_prop_trail_offset: match body {
                DefaultAirship => Some(8.5),
                Submarine => Some(3.5),
                _ => None,
            },
            bone2_prop_trail_offset: match body {
                DefaultAirship => Some(8.5),
                _ => None,
            },
        }
    }
}
