use bevy::{
	prelude::*,
	render::{extract_resource::ExtractResource, Extract},
};

use super::compute_sequence::ComputeSequence;

pub fn extract_resources(
	mut commands: Commands, main_data: Extract<Option<Res<ComputeSequence>>>,
	target_data: Option<ResMut<ComputeSequence>>,
) {
	if let Some(main_data) = &*main_data {
		if let Some(mut target_data) = target_data {
			if main_data.is_changed() {
				*target_data = ComputeSequence::extract_resource(&main_data);
			}
		} else {
			commands.insert_resource(ComputeSequence::extract_resource(&main_data));
		}
	}
}
