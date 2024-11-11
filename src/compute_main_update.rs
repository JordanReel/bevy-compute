use bevy::prelude::*;

use super::{
	compute_data_transmission::{ComputeDataTransmission, ComputeMessage},
	ComputeGroupDoneEvent, CopyBufferEvent,
};

pub fn compute_main_update(
	mut copy_buffer_events: EventWriter<CopyBufferEvent>, mut group_done_events: EventWriter<ComputeGroupDoneEvent>,
	transmission: NonSend<ComputeDataTransmission>,
) {
	while let Ok(data) = transmission.receiver.try_recv() {
		match data {
			ComputeMessage::CopyBuffer(event) => {
				copy_buffer_events.send(event);
			}
			ComputeMessage::GroupDone(event) => {
				group_done_events.send(event);
			}
		}
	}
}
