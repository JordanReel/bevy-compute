use bevy::{
	prelude::*,
	render::{extract_resource::ExtractResource, Extract},
};

use super::active_compute_pipeline::ActiveComputePipeline;

pub fn extract_resources(
	mut commands: Commands, main_data: Extract<Option<Res<ActiveComputePipeline>>>,
	target_data: Option<ResMut<ActiveComputePipeline>>,
) {
	if let Some(main_data) = &*main_data {
		if let Some(mut target_data) = target_data {
			if main_data.is_changed() {
				*target_data = ActiveComputePipeline::extract_resource(&main_data);
			}
		} else {
			commands.insert_resource(ActiveComputePipeline::extract_resource(&main_data));
		}
	}
}
