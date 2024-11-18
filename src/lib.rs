mod compute_bind_groups;
mod compute_data_transmission;
mod compute_main_setup;
mod compute_node;
mod compute_render_setup;
pub mod compute_sequence;
mod extract_resources;
mod parse_render_messages;
mod queue_bind_group;
pub mod shader_buffer_set;
mod swap_sprite_buffers;

use std::{sync::mpsc::sync_channel, time::Duration};

use bevy::{
	prelude::*,
	render::{Render, RenderApp, RenderSet},
};
use compute_data_transmission::ComputeDataTransmission;
use compute_main_setup::compute_main_setup;
use compute_render_setup::compute_render_setup;
use compute_sequence::{ComputeSequence, ComputeTask};
use extract_resources::extract_resources;
use parse_render_messages::parse_render_messages;
use queue_bind_group::queue_bind_group;
use shader_buffer_set::ShaderBufferSetPlugin;
use swap_sprite_buffers::swap_sprite_buffers;

use crate::shader_buffer_set::ShaderBufferHandle;

pub struct BevyComputePlugin;

impl Plugin for BevyComputePlugin {
	fn build(&self, app: &mut App) {
		let (sender, receiver) = sync_channel(16);

		app
			.add_plugins(ShaderBufferSetPlugin)
			.insert_non_send_resource(ComputeDataTransmission { sender, receiver })
			.add_systems(Update, compute_main_setup)
			.add_systems(First, parse_render_messages.run_if(resource_exists::<ComputeSequence>))
			.add_systems(Update, swap_sprite_buffers.run_if(resource_exists::<ComputeSequence>))
			.add_event::<StartComputeEvent>()
			.add_event::<CopyBufferEvent>()
			.add_event::<ComputeGroupDoneEvent>();

		let render_app = app.sub_app_mut(RenderApp);
		render_app
			.add_systems(ExtractSchedule, extract_resources)
			.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue).run_if(resource_exists::<ComputeSequence>))
			.add_systems(Render, compute_render_setup.run_if(resource_added::<ComputeSequence>));
	}
}

#[derive(Event)]
pub struct StartComputeEvent {
	pub tasks: Vec<ComputeTask>,
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

#[derive(Component)]
pub struct DoubleBufferedSprite(pub ShaderBufferHandle);
