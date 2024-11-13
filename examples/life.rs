extern crate bevy_compute;

use std::num::NonZeroU32;

use bevy::{
	prelude::*,
	render::render_resource::{StorageTextureAccess, TextureFormat},
};
use bevy_compute::{
	active_compute_pipeline::{ComputePipelineGroup, PipelineData, PipelineStep},
	shader_buffer_set::{ShaderBufferHandle, ShaderBufferSet},
	BevyComputePlugin, StartComputeEvent,
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/game_of_life.wgsl";

const DISPLAY_FACTOR: u32 = 4;
const SIZE: (u32, u32) = (1280 / DISPLAY_FACTOR, 720 / DISPLAY_FACTOR);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::BLACK))
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						resolution: ((SIZE.0 * DISPLAY_FACTOR) as f32, (SIZE.1 * DISPLAY_FACTOR) as f32).into(),
						// uncomment for unthrottled FPS
						// present_mode: bevy::window::PresentMode::AutoNoVsync,
						..default()
					}),
					..default()
				})
				.set(ImagePlugin::default_nearest()),
			BevyComputePlugin,
		))
		.add_systems(Startup, setup)
		.add_systems(Update, switch_textures)
		.run();
}

fn setup(
	mut commands: Commands, mut buffer_set: ResMut<ShaderBufferSet>, mut images: ResMut<Assets<Image>>,
	mut start_compute_events: EventWriter<StartComputeEvent>,
) {
	let image0 = buffer_set.add_write_texture(
		&mut images,
		SIZE.0,
		SIZE.1,
		TextureFormat::R32Float,
		&0.0f32.to_ne_bytes(),
		StorageTextureAccess::ReadOnly,
		Some((0, 0)),
	);
	let image1 = buffer_set.add_write_texture(
		&mut images,
		SIZE.0,
		SIZE.1,
		TextureFormat::R32Float,
		&0.0f32.to_ne_bytes(),
		StorageTextureAccess::WriteOnly,
		Some((0, 1)),
	);

	commands.insert_resource(LifeBuffers { image0, image1 });

	commands.spawn(SpriteBundle {
		sprite: Sprite { custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)), ..default() },
		texture: buffer_set.image_handle(image0).unwrap(),
		transform: Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),
		..default()
	});
	commands.spawn(Camera2dBundle::default());

	start_compute_events.send(StartComputeEvent {
		groups: vec![
			ComputePipelineGroup {
				label: Some("Init".to_owned()),
				iterations: NonZeroU32::new(1),
				steps: vec![PipelineStep {
					max_frequency: None,
					pipeline_data: PipelineData::RunShader {
						shader: SHADER_ASSET_PATH.to_owned(),
						entry_point: "init".to_owned(),
						x_workgroup_count: SIZE.0 / WORKGROUP_SIZE,
						y_workgroup_count: SIZE.1 / WORKGROUP_SIZE,
						z_workgroup_count: 1,
					},
				}],
			},
			ComputePipelineGroup {
				label: Some("Update".to_owned()),
				iterations: None,
				steps: vec![PipelineStep {
					max_frequency: None,
					pipeline_data: PipelineData::RunShader {
						shader: SHADER_ASSET_PATH.to_owned(),
						entry_point: "update".to_owned(),
						x_workgroup_count: SIZE.0 / WORKGROUP_SIZE,
						y_workgroup_count: SIZE.1 / WORKGROUP_SIZE,
						z_workgroup_count: 1,
					},
				}],
			},
		],
		iteration_buffer: None,
	});
}

fn switch_textures(
	buffers: Res<LifeBuffers>, mut sprite: Query<&mut Handle<Image>, With<Sprite>>,
	mut buffer_set: ResMut<ShaderBufferSet>,
) {
	let image0 = buffer_set.image_handle(buffers.image0).unwrap();
	let image1 = buffer_set.image_handle(buffers.image1).unwrap();
	let mut sprite = sprite.single_mut();
	if *sprite == image0 {
		*sprite = image1;
		buffer_set.set_storage_texture_access(buffers.image1, StorageTextureAccess::ReadOnly);
		buffer_set.set_storage_texture_access(buffers.image0, StorageTextureAccess::WriteOnly);
	} else {
		*sprite = image0;
		buffer_set.set_storage_texture_access(buffers.image0, StorageTextureAccess::ReadOnly);
		buffer_set.set_storage_texture_access(buffers.image1, StorageTextureAccess::WriteOnly);
	}
	buffer_set.swap_buffers(buffers.image0, buffers.image1);
}

#[derive(Resource)]
struct LifeBuffers {
	image0: ShaderBufferHandle,
	image1: ShaderBufferHandle,
}
