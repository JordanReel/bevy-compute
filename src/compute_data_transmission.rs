use std::sync::mpsc::{Receiver, SyncSender};

use super::{ComputeGroupDoneEvent, CopyBufferEvent};
use crate::shader_buffer_set::ShaderBufferHandle;

pub struct ComputeDataTransmission {
	pub sender: SyncSender<ComputeMessage>,
	pub receiver: Receiver<ComputeMessage>,
}

pub enum ComputeMessage {
	CopyBuffer(CopyBufferEvent),
	GroupDone(ComputeGroupDoneEvent),
	SwapBuffers(ShaderBufferHandle),
}
