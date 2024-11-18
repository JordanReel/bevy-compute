use std::{num::NonZeroU32, sync::mpsc::SyncSender};

use bevy::{prelude::*, render::extract_resource::ExtractResource};

use super::compute_data_transmission::ComputeMessage;
use crate::shader_buffer_set::ShaderBufferHandle;

#[derive(Resource, Clone, ExtractResource)]
pub struct ComputeSequence {
	pub sender: SyncSender<ComputeMessage>,
	pub tasks: Vec<ComputeTask>,
	pub iteration_buffer: Option<ShaderBufferHandle>,
}

#[derive(Clone)]
pub struct ComputeTask {
	pub label: Option<String>,
	pub iterations: Option<NonZeroU32>,
	pub steps: Vec<ComputeStep>,
}

#[derive(Clone)]
pub struct ComputeStep {
	pub max_frequency: Option<NonZeroU32>,
	pub action: ComputeAction,
}

#[derive(Clone)]
pub enum ComputeAction {
	RunShader {
		shader: String,
		entry_point: String,
		x_workgroup_count: u32,
		y_workgroup_count: u32,
		z_workgroup_count: u32,
	},
	CopyTexture {
		src: ShaderBufferHandle,
		dst: ShaderBufferHandle,
	},
	CopyBuffer {
		src: ShaderBufferHandle,
	},
	SwapBuffers {
		buffer: ShaderBufferHandle,
	},
}
