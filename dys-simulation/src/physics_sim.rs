use std::sync::mpsc::Receiver;
use rapier3d::glamx::vec3;
use rapier3d::prelude::*;
use rapier3d::parry::query::DefaultQueryDispatcher;

pub struct PhysicsSim {
    gravity: Vec3,
    integration_params: IntegrationParameters,
    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: ChannelEventCollector,
    collision_event_recv: Receiver<CollisionEvent>,
    contact_force_event_recv: Receiver<ContactForceEvent>,
}

impl PhysicsSim {
    pub fn new(ticks_per_second: u32) -> PhysicsSim {
        let integration_params = IntegrationParameters {
            dt: 1.0 / (ticks_per_second as f32),
            max_ccd_substeps: 10,
            ..Default::default()
        };

        let (collision_send, collision_recv) = std::sync::mpsc::channel();
        let (contact_force_send, contact_force_recv) = std::sync::mpsc::channel();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        PhysicsSim {
            gravity: vec3(0.0, -9.81, 0.0),
            integration_params,
            pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler,
            collision_event_recv: collision_recv,
            contact_force_event_recv: contact_force_recv,
        }
    }

    pub fn sets_mut(&mut self) -> (&mut RigidBodySet, &mut ColliderSet) {
        (&mut self.rigid_body_set, &mut self.collider_set)
    }

    pub fn sets(&self) -> (&RigidBodySet, &ColliderSet) {
        (&self.rigid_body_set, &self.collider_set)
    }

    pub fn collision_events(&mut self) -> &mut Receiver<CollisionEvent> {
        &mut self.collision_event_recv
    }

    pub fn contact_force_events(&mut self) -> &mut Receiver<ContactForceEvent> {
        &mut self.contact_force_event_recv
    }

    pub fn query_pipeline<'s>(&'s mut self, query_filter: QueryFilter<'s>) -> QueryPipeline<'s> {
        self.broad_phase.as_query_pipeline(
            &DefaultQueryDispatcher,
            &self.rigid_body_set,
            &self.collider_set,
            query_filter
        )
    }

    pub fn tick(&mut self) {
        self.pipeline.step(
            self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        )
    }

    pub fn gravity_y(&self) -> f32 {
        self.gravity.y
    }
}