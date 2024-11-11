use std::{
	fmt::{Display, Formatter},
	sync::mpsc::channel,
};

use bevy::{
	prelude::*,
	render::{
		extract_resource::ExtractResource,
		render_asset::{RenderAssetUsages, RenderAssets},
		render_resource::{
			encase::private::{WriteInto, Writer},
			BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
			BufferBindingType, BufferDescriptor, BufferInitDescriptor, BufferUsages, Extent3d, Maintain, MapMode,
			ShaderStages, ShaderType, StorageBuffer, StorageTextureAccess, TextureDimension, TextureFormat,
			TextureSampleType, TextureUsages, TextureViewDimension,
		},
		renderer::{RenderContext, RenderDevice, RenderQueue},
		texture::GpuImage,
		Extract, RenderApp,
	},
	utils::HashMap,
};

#[derive(Clone)]
enum ShaderBufferStorage {
	Storage { buffer: Buffer, readonly: bool },
	Uniform(Buffer),
	Texture(Handle<Image>),
	StorageTexture(Handle<Image>),
}

#[derive(Clone)]
pub struct ShaderBufferInfo {
	binding: Option<(u32, u32)>,
	id: u32,
	storage: ShaderBufferStorage,
}

impl ShaderBufferInfo {
	fn new_storage_uninit(
		render_device: &RenderDevice, size: u32, usage: BufferUsages, binding: Option<(u32, u32)>, id: u32, readonly: bool,
	) -> Self {
		let buffer = render_device.create_buffer(&BufferDescriptor {
			label: None,
			size: size as u64,
			usage,
			mapped_at_creation: false,
		});
		Self { binding, id, storage: ShaderBufferStorage::Storage { buffer, readonly } }
	}

	fn new_storage_zeroed(
		render_device: &RenderDevice, size: u32, usage: BufferUsages, binding: Option<(u32, u32)>, id: u32, readonly: bool,
	) -> Self {
		let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
			label: None,
			contents: &vec![0u8; size as usize],
			usage,
		});
		Self { binding, id, storage: ShaderBufferStorage::Storage { buffer, readonly } }
	}

	fn new_storage_init<T: ShaderType + WriteInto + Default>(
		render_device: &RenderDevice, render_queue: &RenderQueue, data: T, usage: BufferUsages,
		binding: Option<(u32, u32)>, id: u32, readonly: bool,
	) -> Self {
		let mut buffer = StorageBuffer::default();
		buffer.set(data);
		buffer.add_usages(usage);
		buffer.write_buffer(&render_device, &render_queue);
		let buffer = buffer.buffer().unwrap().clone();
		Self { binding, id, storage: ShaderBufferStorage::Storage { buffer, readonly } }
	}

	fn new_uniform_init<T: ShaderType + WriteInto + Default>(
		render_device: &RenderDevice, render_queue: &RenderQueue, data: T, usage: BufferUsages,
		binding: Option<(u32, u32)>, id: u32,
	) -> Self {
		let mut buffer = StorageBuffer::default();
		buffer.set(data);
		buffer.add_usages(usage);
		buffer.write_buffer(&render_device, &render_queue);
		let buffer = buffer.buffer().unwrap().clone();
		Self { binding, id, storage: ShaderBufferStorage::Uniform(buffer) }
	}

	fn new_write_texture(
		images: &mut Assets<Image>, width: u32, height: u32, binding: Option<(u32, u32)>, id: u32,
	) -> Self {
		let mut image = Image::new_fill(
			Extent3d { width: width, height: height, depth_or_array_layers: 1 },
			TextureDimension::D2,
			&[255, 0, 0, 255],
			TextureFormat::Rgba8Unorm,
			RenderAssetUsages::RENDER_WORLD,
		);
		image.texture_descriptor.usage =
			TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
		let image = images.add(image);
		Self { binding, id, storage: ShaderBufferStorage::StorageTexture(image) }
	}

	fn new_read_write_texture(
		images: &mut Assets<Image>, width: u32, height: u32, read_binding: Option<(u32, u32)>, read_id: u32,
		write_binding: Option<(u32, u32)>, write_id: u32,
	) -> (Self, Self) {
		let mut image = Image::new_fill(
			Extent3d { width: width, height: height, depth_or_array_layers: 1 },
			TextureDimension::D2,
			&[0, 0, 0, 0],
			TextureFormat::Rgba8Unorm,
			RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
		);
		image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
		let read_image = images.add(image);
		let mut image = Image::new_fill(
			Extent3d { width: width, height: height, depth_or_array_layers: 1 },
			TextureDimension::D2,
			&[0, 0, 0, 0],
			TextureFormat::Rgba8Unorm,
			RenderAssetUsages::RENDER_WORLD,
		);
		image.texture_descriptor.usage =
			TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING;
		let write_image = images.add(image);
		(
			Self { binding: read_binding, id: read_id, storage: ShaderBufferStorage::Texture(read_image) },
			Self { binding: write_binding, id: write_id, storage: ShaderBufferStorage::StorageTexture(write_image) },
		)
	}

	fn bind_group_entry<'a>(&'a self, gpu_images: &'a RenderAssets<GpuImage>) -> Option<BindGroupEntry<'a>> {
		if let Some((_, binding)) = self.binding {
			match &self.storage {
				ShaderBufferStorage::Storage { buffer, readonly: _ } => {
					Some(BindGroupEntry { binding, resource: buffer.as_entire_binding() })
				}
				ShaderBufferStorage::Uniform(buffer) => Some(BindGroupEntry { binding, resource: buffer.as_entire_binding() }),
				ShaderBufferStorage::Texture(image) => {
					let image = gpu_images.get(image).unwrap();
					Some(BindGroupEntry { binding, resource: BindingResource::TextureView(&image.texture_view) })
				}
				ShaderBufferStorage::StorageTexture(image) => {
					let image = gpu_images.get(image).unwrap();
					Some(BindGroupEntry { binding, resource: BindingResource::TextureView(&image.texture_view) })
				}
			}
		} else {
			None
		}
	}

	fn bind_group_layout_entry(&self) -> Option<BindGroupLayoutEntry> {
		if let Some((_, binding)) = self.binding {
			Some(BindGroupLayoutEntry {
				binding,
				visibility: ShaderStages::COMPUTE,
				ty: match &self.storage {
					ShaderBufferStorage::Storage { buffer: _, readonly } => BindingType::Buffer {
						ty: BufferBindingType::Storage { read_only: *readonly },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					ShaderBufferStorage::Uniform(_) => {
						BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }
					}
					ShaderBufferStorage::Texture(_) => BindingType::Texture {
						sample_type: TextureSampleType::Float { filterable: true },
						view_dimension: TextureViewDimension::D2,
						multisampled: false,
					},
					ShaderBufferStorage::StorageTexture(_) => BindingType::StorageTexture {
						access: StorageTextureAccess::WriteOnly,
						format: TextureFormat::Rgba8Unorm,
						view_dimension: TextureViewDimension::D2,
					},
				},
				count: None,
			})
		} else {
			None
		}
	}

	fn image_handle(&self) -> Option<Handle<Image>> {
		if let ShaderBufferStorage::Texture(image) = &self.storage {
			Some(image.clone())
		} else if let ShaderBufferStorage::StorageTexture(image) = &self.storage {
			Some(image.clone())
		} else {
			None
		}
	}

	fn set_buffer<T: ShaderType + WriteInto>(data: T, buffer: &Buffer, render_queue: &RenderQueue) {
		let mut bytes = Vec::new();
		let mut writer = Writer::new(&data, &mut bytes, 0).unwrap();
		data.write_into(&mut writer);
		render_queue.write_buffer(buffer, 0, bytes.as_ref());
	}

	fn set<T: ShaderType + WriteInto>(&self, data: T, render_queue: &RenderQueue) {
		if let ShaderBufferStorage::Storage { buffer, readonly: _ } = &self.storage {
			Self::set_buffer(data, buffer, render_queue);
		} else if let ShaderBufferStorage::Uniform(buffer) = &self.storage {
			Self::set_buffer(data, buffer, render_queue);
		} else {
			panic!("Tried to set data on a buffer that isn't a storage or uniform buffer");
		}
	}
}

#[derive(Resource, Clone, ExtractResource)]
pub struct ShaderBufferSet {
	unbound_buffers: HashMap<u32, ShaderBufferInfo>,
	bound_buffers: Vec<Vec<ShaderBufferInfo>>,
	next_id: u32,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum ShaderBufferHandle {
	Bound { group: u32, binding: u32, id: u32 },
	Unbound { id: u32 },
}

impl Display for ShaderBufferHandle {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ShaderBufferHandle::Bound { group, binding, id } => {
				write!(f, "{{ group({}), binding({}), id({}) }}", group, binding, id)
			}
			ShaderBufferHandle::Unbound { id } => write!(f, "{{ id({}) }}", id),
		}
	}
}

fn bind_group_layout(buffers: &Vec<ShaderBufferInfo>, device: &RenderDevice) -> BindGroupLayout {
	device.create_bind_group_layout(
		None,
		buffers.iter().filter_map(|buffer| buffer.bind_group_layout_entry()).collect::<Vec<_>>().as_slice(),
	)
}

impl ShaderBufferSet {
	fn new() -> Self { Self { unbound_buffers: HashMap::new(), bound_buffers: Vec::new(), next_id: 0 } }

	pub fn add_storage_uninit(
		&mut self, render_device: &RenderDevice, size: u32, usage: BufferUsages, binding: Option<(u32, u32)>,
		readonly: bool,
	) -> ShaderBufferHandle {
		self.store_buffer(
			binding,
			ShaderBufferInfo::new_storage_uninit(render_device, size, usage, binding, self.next_id, readonly),
		)
	}

	pub fn add_storage_zeroed(
		&mut self, render_device: &RenderDevice, size: u32, usage: BufferUsages, binding: Option<(u32, u32)>,
		readonly: bool,
	) -> ShaderBufferHandle {
		self.store_buffer(
			binding,
			ShaderBufferInfo::new_storage_zeroed(render_device, size, usage, binding, self.next_id, readonly),
		)
	}

	pub fn add_storage_init<T: ShaderType + WriteInto + Default>(
		&mut self, render_device: &RenderDevice, render_queue: &RenderQueue, data: T, usage: BufferUsages,
		binding: Option<(u32, u32)>, readonly: bool,
	) -> ShaderBufferHandle {
		self.store_buffer(
			binding,
			ShaderBufferInfo::new_storage_init(render_device, render_queue, data, usage, binding, self.next_id, readonly),
		)
	}

	pub fn add_uniform_init<T: ShaderType + WriteInto + Default>(
		&mut self, render_device: &RenderDevice, render_queue: &RenderQueue, data: T, usage: BufferUsages,
		binding: Option<(u32, u32)>,
	) -> ShaderBufferHandle {
		self.store_buffer(
			binding,
			ShaderBufferInfo::new_uniform_init(render_device, render_queue, data, usage, binding, self.next_id),
		)
	}

	pub fn add_write_texture(
		&mut self, images: &mut Assets<Image>, width: u32, height: u32, binding: Option<(u32, u32)>,
	) -> ShaderBufferHandle {
		self.store_buffer(binding, ShaderBufferInfo::new_write_texture(images, width, height, binding, self.next_id))
	}

	pub fn add_read_write_texture(
		&mut self, images: &mut Assets<Image>, width: u32, height: u32, read_binding: Option<(u32, u32)>,
		write_binding: Option<(u32, u32)>,
	) -> (ShaderBufferHandle, ShaderBufferHandle) {
		let (read, write) = ShaderBufferInfo::new_read_write_texture(
			images,
			width,
			height,
			read_binding,
			self.next_id,
			write_binding,
			self.next_id + 1,
		);
		(self.store_buffer(read_binding, read), self.store_buffer(write_binding, write))
	}

	pub fn bind_groups(&self, device: &RenderDevice, gpu_images: &RenderAssets<GpuImage>) -> Vec<BindGroup> {
		self
			.bound_buffers
			.iter()
			.map(|buffers| {
				device.create_bind_group(
					None,
					&bind_group_layout(buffers, &device),
					buffers.iter().filter_map(|buffer| buffer.bind_group_entry(gpu_images)).collect::<Vec<_>>().as_slice(),
				)
			})
			.collect()
	}

	pub fn bind_group_layouts(&self, device: &RenderDevice) -> Vec<BindGroupLayout> {
		self.bound_buffers.iter().map(|buffers| bind_group_layout(buffers, device)).collect()
	}

	pub fn delete_buffer(&mut self, handle: ShaderBufferHandle, images: &mut Assets<Image>) {
		if let Some(buffer) = self.remove_buffer(handle) {
			match &buffer.storage {
				ShaderBufferStorage::Storage { buffer, .. } => buffer.destroy(),
				ShaderBufferStorage::Uniform(buffer) => buffer.destroy(),
				ShaderBufferStorage::Texture(image) => {
					images.remove(image);
				}
				ShaderBufferStorage::StorageTexture(image) => {
					images.remove(image);
				}
			}
		}
	}

	pub fn image_handle(&self, handle: ShaderBufferHandle) -> Option<Handle<Image>> {
		if let Some(buffer) = self.get_buffer(handle) {
			buffer.image_handle()
		} else {
			None
		}
	}

	pub fn set_buffer<T: ShaderType + WriteInto>(
		&mut self, handle: ShaderBufferHandle, data: T, render_queue: &RenderQueue,
	) {
		if let Some(buffer) = self.get_buffer(handle) {
			buffer.set(data, render_queue);
		} else {
			panic!("Tried to set data on a non-existent buffer");
		}
	}

	pub fn copy_texture(
		&self, src_handle: ShaderBufferHandle, dst_handle: ShaderBufferHandle, render_context: &mut RenderContext,
		images: &RenderAssets<GpuImage>,
	) {
		if let (Some(src), Some(dst)) = (self.get_buffer(src_handle), self.get_buffer(dst_handle)) {
			if let (Some(src), Some(dst)) = (src.image_handle(), dst.image_handle()) {
				if let (Some(src), Some(dst)) = (images.get(&src), images.get(&dst)) {
					let encoder = render_context.command_encoder();
					encoder.copy_texture_to_texture(
						src.texture.as_image_copy(),
						dst.texture.as_image_copy(),
						Extent3d { width: src.texture.width(), height: src.texture.height(), depth_or_array_layers: 1 },
					);
				} else {
					panic!(
						"Tried to copy from texture {} to texture {} when at least one of them doesn't exist on the GPU",
						src_handle, dst_handle
					);
				}
			} else {
				panic!(
					"Tried to copy from texture {} to texture {} when at least one of them isn't a texture",
					src_handle, dst_handle
				);
			}
		} else {
			panic!(
				"Tried to copy from texture {} to texture {} when at least one of them doesn't exist",
				src_handle, dst_handle
			);
		}
	}

	fn store_buffer(&mut self, binding: Option<(u32, u32)>, buffer: ShaderBufferInfo) -> ShaderBufferHandle {
		let handle = if let Some((group, binding)) = binding {
			if binding as usize >= self.bound_buffers.len() {
				self.bound_buffers.resize(binding as usize + 1, Vec::new());
			}
			self.bound_buffers[group as usize].push(buffer);
			ShaderBufferHandle::Bound { group, binding, id: self.next_id }
		} else {
			self.unbound_buffers.insert(self.next_id, buffer);
			ShaderBufferHandle::Unbound { id: self.next_id }
		};
		self.next_id += 1;
		handle
	}

	fn remove_buffer(&mut self, handle: ShaderBufferHandle) -> Option<ShaderBufferInfo> {
		match handle {
			ShaderBufferHandle::Bound { group, id, .. } => {
				if let Some(buffers) = self.bound_buffers.get_mut(group as usize) {
					if let Some(index) = buffers.iter().position(|buffer| buffer.id == id) {
						return Some(buffers.remove(index));
					}
				}
				None
			}
			ShaderBufferHandle::Unbound { id } => self.unbound_buffers.remove(&id),
		}
	}

	fn get_buffer(&self, handle: ShaderBufferHandle) -> Option<ShaderBufferInfo> {
		match handle {
			ShaderBufferHandle::Bound { group, id, .. } => {
				self.bound_buffers[group as usize].iter().find(|buffer| buffer.id == id).cloned()
			}
			ShaderBufferHandle::Unbound { id } => self.unbound_buffers.get(&id).cloned(),
		}
	}
}

fn extract_resources(mut commands: Commands, buffers: Extract<Option<Res<ShaderBufferSet>>>) {
	if let Some(buffers) = &*buffers {
		commands.insert_resource(ShaderBufferSet::extract_resource(&buffers));
	}
}

#[derive(Resource)]
pub struct ShaderBufferRenderSet {
	copy_buffers: HashMap<ShaderBufferHandle, Buffer>,
}

impl ShaderBufferRenderSet {
	fn new() -> Self { Self { copy_buffers: HashMap::new() } }

	pub fn create_copy_buffer(&mut self, handle: ShaderBufferHandle, buffers: &ShaderBufferSet, device: &RenderDevice) {
		if self.copy_buffers.contains_key(&handle) {
			panic!("Tried to create a copy buffer for {}, which already has one", handle);
		}
		if let Some(src) = buffers.get_buffer(handle) {
			if let ShaderBufferStorage::Storage { buffer: src, .. } = &src.storage {
				let dst = ShaderBufferInfo::new_storage_uninit(
					device,
					src.size() as u32,
					BufferUsages::COPY_DST | BufferUsages::MAP_READ,
					None,
					0,
					false,
				);
				if let ShaderBufferStorage::Storage { buffer: dst, .. } = dst.storage {
					self.copy_buffers.insert(handle, dst);
				} else {
					panic!("Tried to create a copy buffer for {}, but somehow it ended up as a non-storage buffer", handle);
				}
			} else {
				panic!("Tried to create a copy buffer for {}, which is not a storage buffer", handle);
			}
		} else {
			panic!("Tried to create a copy buffer for {}, which does not exist", handle);
		}
	}

	pub fn remove_copy_buffer(&mut self, handle: ShaderBufferHandle) {
		if let Some(buffer) = self.copy_buffers.get(&handle) {
			buffer.destroy();
			self.copy_buffers.remove(&handle);
		} else {
			panic!("Tried to remove copy buffer for {}, but it doesn't have one", handle);
		}
	}

	pub fn copy_to_copy_buffer(
		&self, handle: ShaderBufferHandle, buffers: &ShaderBufferSet, context: &mut RenderContext,
	) {
		if let Some(src) = buffers.get_buffer(handle) {
			if let ShaderBufferStorage::Storage { buffer: src, .. } = &src.storage {
				if let Some(dst) = self.copy_buffers.get(&handle) {
					let encoder = context.command_encoder();
					encoder.copy_buffer_to_buffer(&src, 0, &dst, 0, src.size());
				} else {
					panic!("Tried to copy {} to it's copy buffer, but it doesn't yet have one", handle);
				}
			} else {
				panic!("Tried to copy from buffer {}, which is not a storage buffer", handle);
			}
		} else {
			panic!("Tried to copy from buffer {}, which doesn't exist", handle);
		}
	}

	pub fn copy_from_copy_buffer_to_vec(&self, handle: ShaderBufferHandle, device: &RenderDevice) -> Vec<u8> {
		if let Some(buffer) = self.copy_buffers.get(&handle) {
			let buffer_slice = buffer.slice(..);
			let (sender, receiver) = channel();
			buffer_slice.map_async(MapMode::Read, move |result| {
				sender.send(result).unwrap();
			});
			device.poll(Maintain::Wait);
			receiver.recv().unwrap().unwrap();
			let result = buffer_slice.get_mapped_range().to_vec();
			buffer.unmap();
			result
		} else {
			panic!("Tried to copy from buffer {} to vec when it has not yet been copied to a copy buffer", handle);
		}
	}
}

pub struct ShaderBufferSetPlugin;

impl Plugin for ShaderBufferSetPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(ShaderBufferSet::new());
		app
			.sub_app_mut(RenderApp)
			.add_systems(ExtractSchedule, extract_resources)
			.insert_resource(ShaderBufferRenderSet::new());
	}
}
