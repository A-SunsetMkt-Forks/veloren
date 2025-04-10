use super::{
    super::{Animation, vek::*},
    BirdMediumSkeleton, SkeletonAttr,
};
use common::{states::utils::StageSection, util::Dir};

pub struct ShootAnimation;

type ShootAnimationDependency = (Vec3<f32>, f32, Option<StageSection>, f32, Dir, bool);

impl Animation for ShootAnimation {
    type Dependency<'a> = ShootAnimationDependency;
    type Skeleton = BirdMediumSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"bird_medium_shoot\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "bird_medium_shoot"))]
    fn update_skeleton_inner<'a>(
        skeleton: &Self::Skeleton,
        (velocity, global_time, stage_section, timer, look_dir, on_ground): Self::Dependency<'_>,
        anim_time: f32,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        let (movement1base, movement3, twitch) = match stage_section {
            Some(StageSection::Buildup) => (anim_time.powf(0.25), 0.0, 0.0),
            Some(StageSection::Recover) => (1.0, anim_time.powf(0.25), anim_time),
            _ => (0.0, 0.0, 0.0),
        };

        let pullback = 1.0 - movement3;
        let subtract = global_time - timer;
        let check = subtract - subtract.trunc();
        let mirror = (check - 0.5).signum();
        let twitch2 = mirror * (twitch * 20.0).sin() * pullback;
        let movement1abs = movement1base * pullback;
        let movement1mirror = movement1abs * mirror;

        let wave_slow_cos = (anim_time * 4.5).cos();

        next.chest.position = Vec3::new(
            0.0,
            s_a.chest.0,
            s_a.chest.1 + wave_slow_cos * 0.06 + twitch2 * 0.1,
        );

        next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);
        next.head.orientation =
            Quaternion::rotation_x(movement1abs * 0.5 + look_dir.z * 0.4 + twitch2)
                * Quaternion::rotation_y(movement1mirror * 0.5);

        if on_ground {
            next.chest.position = Vec3::new(
                0.0,
                s_a.chest.0,
                s_a.chest.1 + wave_slow_cos * 0.06 + twitch2 * 0.1 + movement1abs * -3.0,
            );

            next.chest.orientation = Quaternion::rotation_x(movement1abs * 0.1);

            next.tail.position = Vec3::new(0.0, s_a.tail.0, s_a.tail.1);
            next.tail.orientation = Quaternion::rotation_x(-movement1abs * 0.1 + twitch2 * 0.02);

            next.leg_l.orientation = Quaternion::rotation_x(movement1abs * -0.5);
            next.leg_r.orientation = Quaternion::rotation_x(movement1abs * -0.5);
        }
        if velocity.xy().magnitude() < 1.0 {
            next.wing_in_l.orientation = Quaternion::rotation_y(-1.0 + movement1abs * 0.8)
                * Quaternion::rotation_z(0.2 - movement1abs * 0.8);
            next.wing_in_r.orientation = Quaternion::rotation_y(1.0 - movement1abs * 0.8)
                * Quaternion::rotation_z(-0.2 + movement1abs * 0.8);
        };
        next
    }
}
