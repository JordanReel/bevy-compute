use std::{num::NonZeroU32, sync::mpsc::SyncSender};

use bevy::{prelude::*, render::extract_resource::ExtractResource};

use super::compute_data_transmission::ComputeMessage;
use crate::shader_buffer_set::ShaderBufferHandle;

#[derive(Resource, Clone, ExtractResource)]
pub(crate) struct ComputeSequence {
	pub sender: SyncSender<ComputeMessage>,
	pub tasks: Vec<ComputeTask>,
	pub iteration_buffer: Option<ShaderBufferHandle>,
}

/// This describes a compute shader task, which is a set of things it should do every tick, for some number of iterations.
#[derive(Clone)]
pub struct ComputeTask {
	/// The optional label is sent back in the [ComputeTaskDoneEvent](crate::ComputeTaskDoneEvent) when this task is completed. It makes it easier to identify which task was completed.
	pub label: Option<String>,

	/// The number of times to run this task before considering it done. If this isn't provided, it will run forever.
	pub iterations: Option<NonZeroU32>,

	/// The set of steps to execute on each iteration.
	pub steps: Vec<ComputeStep>,
}

/// A compute step is one action to do during a compute task.
#[derive(Clone)]
pub struct ComputeStep {
	/// The max frequency allows you to make it so a step won't run on every iteration. If provided, then this is the maximum number of times it will run per second. For instance, if a max frequency of 30 is given, then it will be at least 1000 / 30 = 16.67 ms between each run. When it's going through the steps, if it hasn't been at least 16.67 ms since the last time it ran, it won't run this time.
	///
	/// Compute shaders can sometimes be rather expensive, and use a lot of GPU resources. Not running them every frame can sometimes be a significant performance improvement. If you have a long-running compute task which is providing a real-time visualization, it can be a useful optimization to say that the steps that update the visuals run at a lower frequency. In the Game of Life example, if the game is running at full speed on a 120 Hz monitor, it can be very difficult to see what's going down, so the example slows it down to 10 Hz.
	pub max_frequency: Option<NonZeroU32>,

	/// This is the actual action to perform.
	pub action: ComputeAction,
}

/// A compute action describes the specific action to take during a compute step.
#[derive(Clone)]
pub enum ComputeAction {
	/// This action runs a specific shader.
	RunShader {
		/// The Bevy asset path to the shader file to run.
		shader: String,

		/// The name of the function to run in that shader file.
		entry_point: String,

		/// The workgroup count in the X dimension.
		x_workgroup_count: u32,

		/// The workgroup count in the Y dimension.
		y_workgroup_count: u32,

		/// The workgroup count in the Z dimension.
		z_workgroup_count: u32,
	},

	/// This action copies a texture buffer to another texture buffer of identical size and format.
	CopyTexture {
		/// The source buffer, to copy from.
		src: ShaderBufferHandle,

		/// The destination buffer, to copy to.
		dst: ShaderBufferHandle,
	},

	/// This action copies the contents of a buffer back to the CPU. When this runs, it will throw a [CopyBufferEvent](crate::CopyBufferEvent), which contains the data. This is fairly slow, and actually takes two iterations to run, because the data must first be copied into an intermediate buffer before being copied to the CPU. It's highly recommended that if this is on a compute task that runs for many iterations, it's run with a max frequency. But keep in mind that because it takes two iterations to run, the frequency with which you will recieve data will be half the specified frequency.
	CopyBuffer {
		/// The buffer to copy out of.
		src: ShaderBufferHandle,
	},

	/// This action swaps a double buffer. The front buffer becomes the back buffer, and vice-versa. This swaps which bindings they use, which buffer's data will be returned on a [CopyBuffer](ComputeAction::CopyBuffer), and if this is a texture, which texture buffer's image handle will be returned on a call to [image_handle](crate::ShaderBufferSet::image_handle).
	SwapBuffers {
		/// The double buffer to swap.
		buffer: ShaderBufferHandle,
	},
}
