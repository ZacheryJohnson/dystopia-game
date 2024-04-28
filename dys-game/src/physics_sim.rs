use crossbeam::channel::Receiver;
use rapier3d::{na::Vector3, prelude::*};

pub struct PhysicsSim {
    gravity: Vector3<f32>,
    integration_params: IntegrationParameters,
    pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: ChannelEventCollector,
    collision_event_recv: Receiver<CollisionEvent>,
    contact_force_event_recv: Receiver<ContactForceEvent>,
}

impl PhysicsSim {
    pub fn new(ticks_per_second: u32) -> PhysicsSim {
        let mut integration_params = IntegrationParameters::default();
        integration_params.dt = 1.0 / (ticks_per_second as f32);

        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        PhysicsSim {
            gravity: vector![0.0, -9.81, 0.0],
            integration_params,
            pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
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

    pub fn query_pipeline(&mut self) -> &QueryPipeline {
        &self.query_pipeline
    }

    pub fn query_pipeline_and_sets(&mut self) -> (&mut QueryPipeline, &mut RigidBodySet, &mut ColliderSet) {
        (&mut self.query_pipeline, &mut self.rigid_body_set, &mut self.collider_set)
    }

    pub fn tick(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        )
    }

    pub fn gravity_y(&self) -> f32 {
        self.gravity.y
    }
}