use bevy::prelude::*;

use super::{
	active_compute_pipeline::ActiveComputePipeline, compute_data_transmission::ComputeDataTransmission,
	StartComputeEvent,
};

pub fn compute_main_setup(
	mut commands: Commands, mut start_events: EventReader<StartComputeEvent>,
	transmission: NonSend<ComputeDataTransmission>,
) {
	if let Some(event) = start_events.read().next() {
		commands.insert_resource(ActiveComputePipeline {
			sender: transmission.sender.clone(),
			groups: event.groups.clone(),
			iteration_buffer: event.iteration_buffer,
		});
		if let Some(_) = start_events.read().next() {
			panic!("Attempted to start multiple compute pipelines at once");
		}
	}
}
