use super::{
    super::{Animation, vek::*},
    BipedLargeSkeleton, SkeletonAttr,
};
use common::{comp::item::ToolKind, states::utils::StageSection};
use core::f32::consts::PI;

pub struct LeapAnimation;

impl Animation for LeapAnimation {
    type Dependency<'a> = (
        Option<ToolKind>,
        Option<ToolKind>,
        Vec3<f32>,
        f32,
        Option<StageSection>,
        Option<&'a str>,
    );
    type Skeleton = BipedLargeSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"biped_large_leapmelee\0";

    #[cfg_attr(feature = "be-dyn-lib", unsafe(export_name = "biped_large_leapmelee"))]
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (active_tool_kind, _second_tool_kind, _velocity, _global_time, stage_section, ability_id): Self::Dependency<'_>,
        anim_time: f32,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let mut next = (*skeleton).clone();

        let (movement1, movement2, movement3, movement4) = match stage_section {
            Some(StageSection::Buildup) => (anim_time, 0.0, 0.0, 0.0),
            Some(StageSection::Movement) => (1.0, anim_time.powf(0.25), 0.0, 0.0),
            Some(StageSection::Action) => (1.0, 1.0, anim_time.powf(0.25), 0.0),
            Some(StageSection::Recover) => (1.0, 1.0, 1.0, anim_time),
            _ => (0.0, 0.0, 0.0, 0.0),
        };

        match active_tool_kind {
            Some(ToolKind::Hammer) => {
                next.hand_l.position = Vec3::new(s_a.hhl.0, s_a.hhl.1, s_a.hhl.2);
                next.hand_l.orientation = Quaternion::rotation_x(s_a.hhl.3);
                next.hand_r.position = Vec3::new(s_a.hhr.0, s_a.hhr.1, s_a.hhr.2);
                next.hand_r.orientation = Quaternion::rotation_x(s_a.hhr.3);
                next.main.position = Vec3::new(0.0, 0.0, 0.0);
                next.main.orientation = Quaternion::rotation_y(0.0) * Quaternion::rotation_z(0.0);

                next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);

                next.control.position = Vec3::new(
                    s_a.hc.0 + movement2 * -10.0 + movement3 * 6.0,
                    s_a.hc.1 + movement2 * 5.0 + movement3 * 7.0,
                    s_a.hc.2 + movement2 * 5.0 + movement3 * -10.0,
                );
                next.control.orientation =
                    Quaternion::rotation_x(s_a.hc.3 + movement2 * PI / 2.0 + movement3 * -2.3)
                        * Quaternion::rotation_y(s_a.hc.4 + movement2 * 1.3)
                        * Quaternion::rotation_z(s_a.hc.5 + movement2 * -1.0 + movement3 * 0.5);
                next.upper_torso.orientation = Quaternion::rotation_x(
                    movement1 * 0.3 + movement2 * 0.3 + movement3 * -0.9 + movement4 * 0.3,
                ) * Quaternion::rotation_z(
                    movement1 * 0.5 + movement2 * 0.2 + movement3 * -0.7,
                );

                next.head.orientation = Quaternion::rotation_x(movement3 * 0.2)
                    * Quaternion::rotation_y(0.0 + movement2 * -0.1)
                    * Quaternion::rotation_z(movement1 * -0.4 + movement2 * -0.2 + movement3 * 0.6);

                //next.hand_l.position = Vec3::new(-12.0 + movement3 * 10.0, 0.0, 0.0);

                next.foot_l.position = Vec3::new(
                    -s_a.foot.0,
                    s_a.foot.1 + movement3 * 13.0,
                    s_a.foot.2 + movement3 * -2.0,
                );
                next.foot_l.orientation = Quaternion::rotation_x(-0.8 + movement3 * 1.7);

                next.foot_r.position = Vec3::new(
                    s_a.foot.0,
                    s_a.foot.1 + 8.0 + movement3 * -13.0,
                    s_a.foot.2 + 5.0 + movement3 * -5.0,
                );
                next.foot_r.orientation = Quaternion::rotation_x(0.9 + movement3 * -1.7);
            },
            Some(ToolKind::Natural) => match ability_id {
                Some("common.abilities.custom.tursus.tusk_bash_leap") => {
                    next.head.position = Vec3::new(
                        0.0,
                        s_a.head.0 + movement3 * 12.0,
                        s_a.head.1 + movement3 * 8.0,
                    );
                    next.upper_torso.position =
                        Vec3::new(0.0, s_a.upper_torso.0, s_a.upper_torso.1 + movement1 * -4.0);
                    next.upper_torso.orientation = Quaternion::rotation_x(
                        movement1 * -0.3 + movement2 * -0.3 + movement3 * 0.2,
                    ) * Quaternion::rotation_z(
                        movement1 * 0.1 + movement2 * 0.2 + movement3 * -0.3,
                    );

                    next.head.orientation = Quaternion::rotation_x(
                        movement1 * -0.1 + movement2 * -0.1 + movement3 * 1.4,
                    );
                    next.foot_l.position = Vec3::new(
                        -s_a.foot.0,
                        s_a.foot.1 + movement3 * 1.0,
                        s_a.foot.2 + movement1 * -0.3 + movement3 * -2.0,
                    );
                    next.foot_l.orientation = Quaternion::rotation_x(
                        movement1 * -0.1 + movement2 * -0.2 + movement3 * 0.7,
                    );

                    next.foot_r.position = Vec3::new(
                        s_a.foot.0,
                        s_a.foot.1 + 1.0 + movement3 * -1.0,
                        s_a.foot.2 - 1.0 + movement1 * -0.3 + movement3 * -2.0,
                    );
                    next.foot_r.orientation = Quaternion::rotation_x(
                        movement1 * -0.1 + movement2 * -0.2 + movement3 * -0.7,
                    );
                    next.shoulder_l.orientation = Quaternion::rotation_x(
                        movement1 * 0.5 + movement2 * 0.1 + movement3 * -1.5,
                    );
                    next.shoulder_r.orientation = Quaternion::rotation_x(
                        movement1 * 0.9 + movement2 * 0.1 + movement3 * -1.5,
                    );
                    next.hand_l.orientation = Quaternion::rotation_x(
                        movement1 * 0.5 + movement2 * 0.1 + movement3 * -1.5,
                    );
                    next.hand_r.orientation = Quaternion::rotation_x(
                        movement1 * 0.9 + movement2 * 0.1 + movement3 * -1.5,
                    );
                },
                _ => {},
            },
            Some(ToolKind::Sword) => match ability_id {
                Some("common.abilities.custom.gigas_fire.lava_leap") => {
                    let move1 = (PI * movement1).sin();
                    let move2 = movement2 * (1.0 - movement3);
                    let move3 = movement3 * (1.0 - movement4.powi(3));

                    next.control.position = Vec3::new(s_a.sc.0, s_a.sc.1, s_a.sc.2);
                    next.control.orientation = Quaternion::rotation_x(s_a.sc.3)
                        * Quaternion::rotation_y(s_a.sc.4)
                        * Quaternion::rotation_z(s_a.sc.5);
                    next.hand_l.position = Vec3::new(s_a.shl.0, s_a.shl.1, s_a.shl.2);
                    next.hand_l.orientation =
                        Quaternion::rotation_x(s_a.shl.3) * Quaternion::rotation_y(s_a.shl.4);
                    next.hand_r.position = Vec3::new(s_a.shr.0, s_a.shr.1, s_a.shr.2);
                    next.hand_r.orientation =
                        Quaternion::rotation_x(s_a.shr.3) * Quaternion::rotation_y(s_a.shr.4);
                    next.main.position = Vec3::new(1.0, 10.0, 0.0);
                    next.main.orientation =
                        Quaternion::rotation_y(0.0) * Quaternion::rotation_z(0.0);

                    next.torso.position += Vec3::new(0.0, -3.0, -3.0) * move1;
                    next.leg_l.position += Vec3::new(0.0, 1.5, 1.5) * move1;
                    next.leg_l.orientation.rotate_x(PI / 3.0 * move1);
                    next.foot_l.position += Vec3::new(0.0, 3.0, 3.0) * move1;
                    next.leg_r.position += Vec3::new(0.0, 1.5, 1.5) * move1;
                    next.leg_r.orientation.rotate_x(PI / 3.0 * move1);
                    next.foot_r.position += Vec3::new(0.0, 3.0, 3.0) * move1;

                    next.torso.orientation.rotate_x(PI / 5.0 * move2);
                    next.torso.position += Vec3::new(0.0, 5.0, 0.0) * move2;
                    next.foot_l.orientation.rotate_x(-PI / 8.0 * move2);
                    next.leg_r.position += Vec3::new(0.0, 5.0, 0.0) * move2;
                    next.leg_r.orientation.rotate_x(PI / 8.0 * move2);
                    next.foot_r.position += Vec3::new(0.0, 2.0, 0.0) * move2;
                    next.foot_r.orientation.rotate_x(-PI / 6.0 * move2);
                    next.shoulder_l.orientation.rotate_x(PI / 2.5 * move2);
                    next.shoulder_r.position += Vec3::new(-3.0, 7.0, 0.0) * move2;
                    next.shoulder_r.orientation.rotate_x(PI / 1.5 * move2);
                    next.shoulder_r.orientation.rotate_z(PI / 4.0 * move2);
                    next.control.position += Vec3::new(-8.0, 0.0, 15.0) * move2;
                    next.control.orientation.rotate_x(PI / 3.0 * move2);
                    next.control.orientation.rotate_z(-PI / 10.0 * move2);
                    next.control_r.position += Vec3::new(13.0, 4.0, -8.0) * move2;
                    next.control_r.orientation.rotate_x(PI / 8.0 * move2);
                    next.control_r.orientation.rotate_z(PI / 3.0 * move2);
                    next.control_l.position += Vec3::new(0.0, 0.0, -7.0) * move2;
                    next.control_l.orientation.rotate_x(PI / 8.0 * move2);

                    next.torso.position += Vec3::new(0.0, -9.0, 0.0) * move3;
                    next.torso.orientation.rotate_x(-PI / 8.0 * move3);
                    next.lower_torso.position += Vec3::new(0.0, 0.0, 1.0) * move3;
                    next.lower_torso.orientation.rotate_x(PI / 8.0 * move3);
                    next.shoulder_r.position += Vec3::new(-3.0, 6.0, 0.0) * move3;
                    next.shoulder_r.orientation.rotate_z(PI / 4.0 * move3);
                    next.shoulder_l.position += Vec3::new(3.0, 6.0, 0.0) * move3;
                    next.shoulder_l.orientation.rotate_z(-PI / 4.0 * move3);
                    next.control.position += Vec3::new(0.0, 0.0, 0.0) * move3;
                    next.control.orientation.rotate_x(-PI / 2.5 * move3);
                },
                Some("common.abilities.adlet.elder.leap") => {
                    next.hand_l.position = Vec3::new(s_a.hhl.0 * 1.5, -s_a.hhl.1, 5.0);
                    next.hand_l.orientation = Quaternion::rotation_x(s_a.hhl.3);
                    next.hand_r.position = Vec3::new(s_a.hhr.0 / 2.0, 12.0, 5.0);
                    next.hand_r.orientation = Quaternion::rotation_x(s_a.hhr.3);
                    next.main.position = Vec3::new(-6.0, 18.0, 4.0);
                    next.main.orientation =
                        Quaternion::rotation_y(0.0) * Quaternion::rotation_z(0.0);
                    next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);
                    next.control.orientation =
                        Quaternion::rotation_x(movement2 * PI / 2.5 + movement3 * -2.3);
                    next.upper_torso.orientation = Quaternion::rotation_x(
                        movement1 * 0.3 + movement2 * 0.3 + movement3 * -0.9 + movement4 * 0.3,
                    ) * Quaternion::rotation_z(
                        movement1 * 0.5 + movement2 * 0.2 + movement3 * -0.7,
                    );

                    next.head.orientation = Quaternion::rotation_x(movement3 * 0.2)
                        * Quaternion::rotation_y(0.0 + movement2 * -0.1)
                        * Quaternion::rotation_z(
                            movement1 * -0.4 + movement2 * -0.2 + movement3 * 0.6,
                        );

                    next.foot_l.position = Vec3::new(
                        -s_a.foot.0,
                        s_a.foot.1 + movement3 * 13.0,
                        s_a.foot.2 + movement3 * -2.0,
                    );
                    next.foot_l.orientation = Quaternion::rotation_x(-0.8 + movement3 * 1.7);

                    next.foot_r.position = Vec3::new(
                        s_a.foot.0,
                        s_a.foot.1 + 8.0 + movement3 * -13.0,
                        s_a.foot.2 + 5.0 + movement3 * -5.0,
                    );
                    next.foot_r.orientation = Quaternion::rotation_x(0.9 + movement3 * -1.7);
                    if ability_id == Some("common.abilities.adlet.elder.leap") {
                        next.second.position = Vec3::new(-2.0, 20.0, 4.0);
                        next.second.orientation =
                            Quaternion::rotation_y(0.0) * Quaternion::rotation_z(0.0);
                    } else {
                        next.second.scale = Vec3::one() * 0.0;
                    }
                },
                _ => {},
            },
            _ => {},
        }

        next
    }
}
