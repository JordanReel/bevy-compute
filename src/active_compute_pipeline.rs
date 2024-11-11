use std::{num::NonZeroU32, sync::mpsc::SyncSender};

use bevy::{prelude::*, render::extract_resource::ExtractResource};

use super::compute_data_transmission::ComputeMessage;
use crate::shader_buffer_set::ShaderBufferHandle;

#[derive(Resource, Clone, ExtractResource)]
pub struct ActiveComputePipeline {
	pub sender: SyncSender<ComputeMessage>,
	pub groups: Vec<ComputePipelineGroup>,
	pub iteration_buffer: Option<ShaderBufferHandle>,
}

#[derive(Clone)]
pub struct ComputePipelineGroup {
	pub label: Option<String>,
	pub iterations: Option<NonZeroU32>,
	pub steps: Vec<PipelineStep>,
}

#[derive(Clone)]
pub struct PipelineStep {
	pub max_frequency: Option<NonZeroU32>,
	pub pipeline_data: PipelineData,
}

#[derive(Clone)]
pub enum PipelineData {
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
}
