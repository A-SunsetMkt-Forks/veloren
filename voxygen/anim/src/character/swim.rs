use super::{
    super::{Animation, vek::*},
    CharacterSkeleton, SkeletonAttr,
};
use common::comp::item::{Hands, ToolKind};
use core::{f32::consts::PI, ops::Mul};

pub struct SwimAnimation;

type SwimAnimationDependency = (
    Option<ToolKind>,
    Option<ToolKind>,
    (Option<Hands>, Option<Hands>),
    Vec3<f32>,
    Vec3<f32>,
    Vec3<f32>,
    f32,
    Vec3<f32>,
);

impl Animation for SwimAnimation {
    type Dependency<'a> = SwimAnimationDependency;
    type Skeleton = CharacterSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"character_swim\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "character_swim"))]

    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (
            active_tool_kind,
            second_tool_kind,
            hands,
            velocity,
            orientation,
            last_ori,
            global_time,
            avg_vel,
        ): Self::Dependency<'_>,
        anim_time: f32,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();
        let avgspeed = Vec2::<f32>::from(avg_vel).magnitude();

        let avgtotal = avg_vel.magnitude();

        let speed = velocity.magnitude();
        *rate = 1.0;
        let tempo = if speed > 0.5 { 1.5 } else { 0.7 };
        let intensity = if speed > 0.5 { 1.0 } else { 0.3 };

        let lab: f32 = 1.0 * tempo;

        let short = (anim_time * lab * 6.0 + PI * 0.9).sin();

        let foot = (anim_time * lab * 6.0 + PI * -0.1).sin();

        let footrotl = ((1.0 / (0.2 + (0.8) * ((anim_time * 6.0 * lab + PI * 1.4).sin()).powi(2)))
            .sqrt())
            * ((anim_time * 6.0 * lab + PI * 1.4).sin());

        let footrotr = ((1.0 / (0.2 + (0.8) * ((anim_time * 6.0 * lab + PI * 0.4).sin()).powi(2)))
            .sqrt())
            * ((anim_time * 6.0 * lab + PI * 0.4).sin());

        let foothoril = (anim_time * 6.0 * lab + PI * 1.4).sin();
        let foothorir = (anim_time * 6.0 * lab + PI * (0.4)).sin();
        let head_look = Vec2::new(
            (global_time + anim_time / 4.0 * (1.0 / tempo))
                .floor()
                .mul(7331.0)
                .sin()
                * 0.2,
            (global_time + anim_time / 4.0 * (1.0 / tempo))
                .floor()
                .mul(1337.0)
                .sin()
                * 0.1,
        );
        let ori: Vec2<f32> = Vec2::from(orientation);
        let last_ori = Vec2::from(last_ori);
        let tilt = if vek::Vec2::new(ori, last_ori)
            .map(|o| o.magnitude_squared())
            .map(|m| m > 0.001 && m.is_finite())
            .reduce_and()
            && ori.angle_between(last_ori).is_finite()
        {
            ori.angle_between(last_ori).min(0.8)
                * last_ori.determine_side(Vec2::zero(), ori).signum()
        } else {
            0.0
        } * 1.3;
        let abstilt = tilt.abs();

        let squash = if abstilt > 0.2 { 0.35 } else { 1.0 }; //condenses the body at strong turns
        next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1 - 1.0 + short * 0.3);
        next.head.orientation =
            Quaternion::rotation_z(head_look.x * 0.3 + short * -0.2 * intensity + tilt * 3.0)
                * Quaternion::rotation_x(
                    (0.4 * head_look.y * (1.0 / intensity)).abs()
                        + 0.45 * intensity
                        + velocity.z * 0.03
                        - (abstilt * 1.8).min(0.0),
                );
        next.head.scale = Vec3::one() * s_a.head_scale;

        next.chest.position = Vec3::new(
            0.0,
            s_a.chest.0,
            -10.0 + s_a.chest.1 + short * 0.3 * intensity,
        );
        next.chest.orientation = Quaternion::rotation_z(short * 0.1 * intensity);

        next.belt.position = Vec3::new(0.0, s_a.belt.0, s_a.belt.1);
        next.belt.orientation = Quaternion::rotation_x(velocity.z.abs() * -0.005 + abstilt * 1.0)
            * Quaternion::rotation_z(short * -0.2 * intensity);

        next.back.position = Vec3::new(0.0, s_a.back.0, s_a.back.1);
        next.back.scale = Vec3::one() * 1.02;

        next.shorts.position = Vec3::new(0.0, s_a.shorts.0, s_a.shorts.1);
        next.shorts.orientation = Quaternion::rotation_x(velocity.z.abs() * -0.005 + abstilt * 1.0)
            * Quaternion::rotation_z(short * -0.3 * intensity);

        next.hand_l.position = Vec3::new(
            -1.0 - s_a.hand.0,
            1.5 + s_a.hand.1 - foot * 2.0 * intensity * squash,
            intensity * 5.0 + s_a.hand.2 + foot * -5.0 * intensity * squash,
        );
        next.hand_l.orientation = Quaternion::rotation_x(1.5 + foot * -1.2 * intensity * squash)
            * Quaternion::rotation_y(0.4 + foot * -0.35);
        next.hand_l.scale = Vec3::one() * 1.04;

        next.hand_r.position = Vec3::new(
            1.0 + s_a.hand.0,
            1.5 + s_a.hand.1 + foot * 2.0 * intensity * squash,
            intensity * 5.0 + s_a.hand.2 + foot * 5.0 * intensity * squash,
        );
        next.hand_r.orientation = Quaternion::rotation_x(1.5 + foot * 1.2 * intensity * squash)
            * Quaternion::rotation_y(-0.4 + foot * -0.35);
        next.hand_r.scale = Vec3::one() * 1.04;

        next.foot_l.position = Vec3::new(
            -s_a.foot.0,
            s_a.foot.1 + foothoril * 1.5 * intensity * squash,
            -10.0 + s_a.foot.2 + footrotl * 3.0 * intensity * squash,
        );
        next.foot_l.orientation =
            Quaternion::rotation_x(-0.8 * squash + footrotl * 0.4 * intensity * squash);

        next.foot_r.position = Vec3::new(
            s_a.foot.0,
            s_a.foot.1 + foothorir * 1.5 * intensity * squash,
            -10.0 + s_a.foot.2 + footrotr * 3.0 * intensity * squash,
        );
        next.foot_r.orientation =
            Quaternion::rotation_x(-0.8 * squash + footrotr * 0.4 * intensity * squash);

        next.shoulder_l.position = Vec3::new(-s_a.shoulder.0, s_a.shoulder.1, s_a.shoulder.2);
        next.shoulder_l.orientation = Quaternion::rotation_x(short * 0.15 * intensity);
        next.shoulder_l.scale = Vec3::one() * 1.1;

        next.shoulder_r.position = Vec3::new(s_a.shoulder.0, s_a.shoulder.1, s_a.shoulder.2);
        next.shoulder_r.orientation = Quaternion::rotation_x(short * -0.15 * intensity);
        next.shoulder_r.scale = Vec3::one() * 1.1;

        next.glider.position = Vec3::new(0.0, 0.0, 10.0);
        next.glider.scale = Vec3::one() * 0.0;

        next.do_tools_on_back(hands, active_tool_kind, second_tool_kind);

        next.lantern.position = Vec3::new(s_a.lantern.0, s_a.lantern.1, s_a.lantern.2);
        next.lantern.scale = Vec3::one() * 0.65;
        next.hold.scale = Vec3::one() * 0.0;

        let switch = if avg_vel.z > 0.0 && avgspeed < 0.5 {
            avgtotal.min(0.5)
        } else {
            avgtotal
        };
        next.torso.position = Vec3::new(0.0, 0.0, 11.0 - avgspeed * 0.55);
        next.torso.orientation = Quaternion::rotation_x(
            (((1.0 / switch) * PI / 2.0 + avg_vel.z * 0.12).min(PI / 2.0) - PI / 2.0)
                + avgspeed * avg_vel.z * -0.003,
        ) * Quaternion::rotation_y(tilt * 2.0)
            * Quaternion::rotation_z(tilt * 3.0);

        next
    }
}
