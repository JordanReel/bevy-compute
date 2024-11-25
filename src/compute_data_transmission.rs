use std::sync::mpsc::{Receiver, SyncSender};

use super::{ComputeTaskDoneEvent, CopyBufferEvent};
use crate::shader_buffer_set::ShaderBufferHandle;

pub struct ComputeDataTransmission {
	pub sender: SyncSender<ComputeMessage>,
	pub receiver: Receiver<ComputeMessage>,
}

pub enum ComputeMessage {
	CopyBuffer(CopyBufferEvent),
	GroupDone(ComputeTaskDoneEvent),
	SwapBuffers(ShaderBufferHandle),
}
