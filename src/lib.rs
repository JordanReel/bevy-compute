pub mod active_compute_pipeline;
mod compute_bind_groups;
mod compute_main_setup;
mod compute_main_update;
mod compute_node;
mod compute_render_setup;
mod compute_data_transmission;
mod extract_resources;
mod queue_bind_group;
pub mod shader_buffer_set;

use std::{sync::mpsc::sync_channel, time::Duration};

use active_compute_pipeline::{ActiveComputePipeline, ComputePipelineGroup};
use bevy::{
	prelude::*,
	render::{Render, RenderApp, RenderSet},
};
use compute_main_setup::compute_main_setup;
use compute_main_update::compute_main_update;
use compute_render_setup::compute_render_setup;
use compute_data_transmission::ComputeDataTransmission;
use extract_resources::extract_resources;
use queue_bind_group::queue_bind_group;
use shader_buffer_set::ShaderBufferSetPlugin;

use crate::shader_buffer_set::ShaderBufferHandle;

pub struct BevyComputePlugin;

impl Plugin for BevyComputePlugin {
	fn build(&self, app: &mut App) {
		let (sender, receiver) = sync_channel(16);

		app
			.add_plugins(ShaderBufferSetPlugin)
			.insert_non_send_resource(ComputeDataTransmission { sender, receiver })
			.add_systems(Update, compute_main_setup)
			.add_systems(Update, compute_main_update.run_if(resource_exists::<ActiveComputePipeline>))
			.add_event::<StartComputeEvent>()
			.add_event::<CopyBufferEvent>()
			.add_event::<ComputeGroupDoneEvent>();

		let render_app = app.sub_app_mut(RenderApp);
		render_app
			.add_systems(ExtractSchedule, extract_resources)
			.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue).run_if(resource_exists::<ActiveComputePipeline>))
			.add_systems(Render, compute_render_setup.run_if(resource_added::<ActiveComputePipeline>));
	}
}

#[derive(Event)]
pub struct StartComputeEvent {
	pub groups: Vec<ComputePipelineGroup>,
	pub iteration_buffer: Option<ShaderBufferHandle>,
}

#[derive(Event)]
pub struct CopyBufferEvent {
	pub buffer: ShaderBufferHandle,
	pub data: Vec<u8>,
}

#[derive(Event)]
pub struct ComputeGroupDoneEvent {
	pub group_finished: usize,
	pub group_finished_label: Option<String>,
	pub time_in_group: Duration,
	pub final_group: bool,
}
