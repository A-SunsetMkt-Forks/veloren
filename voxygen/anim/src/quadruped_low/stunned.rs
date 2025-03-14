use super::{
    super::{Animation, vek::*},
    QuadrupedLowSkeleton, SkeletonAttr,
};
use common::states::utils::StageSection;
//use std::ops::Rem;

pub struct StunnedAnimation;

impl Animation for StunnedAnimation {
    type Dependency<'a> = (f32, f32, Option<StageSection>, f32);
    type Skeleton = QuadrupedLowSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"quadruped_low_stunned\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "quadruped_low_stunned"))]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_velocity, global_time, stage_section, timer): Self::Dependency<'_>,
        anim_time: f32,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let (movement1base, movement2, twitch) = match stage_section {
            Some(StageSection::Buildup) => (anim_time.powf(0.25), 0.0, 0.0),
            Some(StageSection::Recover) => {
                (1.0, anim_time.powf(3.0), ((1.0 - anim_time) * 7.0).sin())
            },
            _ => (0.0, 0.0, 0.0),
        };
        let pullback = 1.0 - movement2;
        let subtract = global_time - timer;
        let check = subtract - subtract.trunc();
        let mirror = (check - 0.5).signum();
        let movement1 = mirror * movement1base * pullback;
        let movement1abs = movement1base * pullback;

        // Center head
        next.head_c_upper.orientation = Quaternion::rotation_x(movement1abs * -0.18)
            * Quaternion::rotation_z(twitch * 0.13 * mirror);

        next.head_c_lower.orientation =
            Quaternion::rotation_x(movement1abs * -0.18) * Quaternion::rotation_y(movement1 * 0.3);

        next.jaw_c.orientation = Quaternion::rotation_x(0.0);

        // Left head
        next.head_l_upper.orientation = Quaternion::rotation_x(movement1abs * -0.18)
            * Quaternion::rotation_z(twitch * 0.13 * mirror);

        next.head_l_lower.orientation =
            Quaternion::rotation_x(movement1abs * -0.18) * Quaternion::rotation_y(movement1 * 0.3);

        next.jaw_l.orientation = Quaternion::rotation_x(0.0);

        // Right head
        next.head_r_upper.orientation = Quaternion::rotation_x(movement1abs * -0.18)
            * Quaternion::rotation_z(twitch * 0.13 * mirror);

        next.head_r_lower.orientation =
            Quaternion::rotation_x(movement1abs * -0.18) * Quaternion::rotation_y(movement1 * 0.3);

        next.jaw_r.orientation = Quaternion::rotation_x(0.0);

        next.chest.orientation =
            Quaternion::rotation_y(movement1 * -0.08) * Quaternion::rotation_z(movement1 * -0.15);

        next.tail_front.orientation =
            Quaternion::rotation_x(0.15) * Quaternion::rotation_z(movement1 * -0.4);
        if s_a.tongue_for_tail {
            next.tail_front.scale = Vec3::one() * 0.1;
            next.tail_rear.scale = Vec3::one() * 0.1;
        } else {
            next.tail_rear.orientation =
                Quaternion::rotation_x(-0.12) * Quaternion::rotation_z(movement1 * -0.4);
        }
        next
    }
}
