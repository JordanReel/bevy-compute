use bevy::{
	ecs::system::SystemState,
	prelude::*,
	render::{
		graph::CameraDriverLabel,
		render_graph::{RenderGraph, RenderLabel},
	},
};

use super::{active_compute_pipeline::ActiveComputePipeline, compute_node::ComputeNode};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct ComputeLabel;

pub fn compute_render_setup(world: &mut World) {
	let mut system_state: SystemState<(ResMut<RenderGraph>, Res<ActiveComputePipeline>)> = SystemState::new(world);
	let (mut render_graph, pipelines) = system_state.get_mut(world);

	render_graph.add_node(ComputeLabel, ComputeNode::new(&pipelines));
	render_graph.add_node_edge(ComputeLabel, CameraDriverLabel);
}
