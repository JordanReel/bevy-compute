use bevy::{
	prelude::*,
	render::{render_asset::RenderAssets, renderer::RenderDevice, texture::GpuImage},
};

use super::compute_bind_groups::ComputeBindGroups;
use crate::shader_buffer_set::ShaderBufferSet;

pub fn queue_bind_group(
	mut commands: Commands, buffers: Res<ShaderBufferSet>, gpu_images: Res<RenderAssets<GpuImage>>,
	render_device: Res<RenderDevice>,
) {
	let bind_groups = buffers.bind_groups(&render_device, &gpu_images);
	commands.insert_resource(ComputeBindGroups(bind_groups));
}
