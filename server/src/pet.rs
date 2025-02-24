use crate::{client::Client, events::shared::update_map_markers};
use common::{
    comp::{
        self, Agent, Alignment, Behavior, BehaviorCapability, Pet, TradingBehavior, anchor::Anchor,
        group::GroupManager,
    },
    uid::Uid,
};
use common_net::msg::ServerGeneral;
use specs::{Entity, Join, WorldExt};
use tracing::{error, warn};

/// Restores a pet retrieved from the database on login, assigning it to its
/// owner
pub fn restore_pet(ecs: &specs::World, pet_entity: Entity, owner: Entity, pet: Pet) {
    tame_pet_internal(ecs, pet_entity, owner, Some(pet));
}

/// Tames a pet, adding to the owner's group and setting its alignment
pub fn tame_pet(ecs: &specs::World, pet_entity: Entity, owner: Entity) {
    tame_pet_internal(ecs, pet_entity, owner, None);
}

fn tame_pet_internal(ecs: &specs::World, pet_entity: Entity, owner: Entity, pet: Option<Pet>) {
    let uids = ecs.read_storage::<Uid>();
    let (owner_uid, pet_uid) = match (uids.get(owner), uids.get(pet_entity)) {
        (Some(uid_owner), Some(uid_pet)) => (*uid_owner, *uid_pet),
        _ => return,
    };
    let mut alignments = ecs.write_storage::<Alignment>();
    let Some(owner_alignment) = alignments.get(owner).copied() else {
        error!("Owner of a pet must have an Alignment");
        return;
    };

    if let Some(Alignment::Owned(existing_owner_uid)) = alignments.get(pet_entity) {
        if *existing_owner_uid != owner_uid {
            warn!("Disallowing taming of pet already owned by another entity");
            return;
        }
    }

    if let Alignment::Owned(owner_alignment_uid) = owner_alignment {
        if owner_alignment_uid != owner_uid {
            error!("Pets cannot be owners of pets");
            return;
        }
    }

    if (
        &ecs.entities(),
        &alignments,
        ecs.read_storage::<Pet>().mask(),
    )
        .join()
        .any(|(_, alignment, _)| matches!(alignment, Alignment::Owned(uid) if *uid == pet_uid))
    {
        error!("Cannot tame entity which owns pets");
        return;
    }

    let _ = alignments.insert(pet_entity, common::comp::Alignment::Owned(owner_uid));

    // Anchor the pet to the player to prevent it de-spawning
    // when its chunk is unloaded if its owner is still logged
    // in
    let _ = ecs
        .write_storage()
        .insert(pet_entity, Anchor::Entity(owner));

    let _ = ecs
        .write_storage()
        .insert(pet_entity, pet.unwrap_or_default());

    // Create an agent for this entity using its body
    if let Some(body) = ecs.read_storage().get(pet_entity) {
        // Pets can trade with their owner
        let mut agent = Agent::from_body(body).with_behavior(
            Behavior::default().maybe_with_capabilities(Some(BehaviorCapability::TRADE)),
        );
        agent.behavior.trading_behavior = TradingBehavior::AcceptFood;
        // Pets shouldn't wander too far from their owner
        agent.psyche.idle_wander_factor = 0.25;
        agent.psyche.aggro_range_multiplier = 0.25;
        agent.patrol_origin = None;
        let _ = ecs.write_storage().insert(pet_entity, agent);
    }

    // Add to group system
    let clients = ecs.read_storage::<Client>();
    let mut group_manager = ecs.write_resource::<GroupManager>();
    let map_markers = ecs.read_storage::<comp::MapMarker>();

    drop(alignments);
    group_manager.new_pet(
        pet_entity,
        owner,
        &mut ecs.write_storage(),
        &ecs.entities(),
        &ecs.read_storage(),
        &uids,
        &mut |entity, group_change| {
            clients
                .get(entity)
                .and_then(|c| {
                    group_change
                        .try_map_ref(|e| uids.get(*e).copied())
                        .map(|g| (g, c))
                })
                .map(|(g, c)| {
                    // Might be unneccessary, but maybe pets can somehow have map
                    // markers in the future
                    update_map_markers(&map_markers, &uids, c, &group_change);
                    c.send_fallible(ServerGeneral::GroupUpdate(g));
                });
        },
    );
}
