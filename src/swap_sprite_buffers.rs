use bevy::prelude::*;

use crate::{shader_buffer_set::ShaderBufferSet, DoubleBufferedSprite};

pub fn swap_sprite_buffers(
	mut sprite: Query<(&mut Sprite, &DoubleBufferedSprite)>, buffer_set: ResMut<ShaderBufferSet>,
) {
	for (mut sprite, DoubleBufferedSprite(buffer_handle)) in sprite.iter_mut() {
		let image = buffer_set.image_handle(*buffer_handle).unwrap_or_else(|| {
			panic!(
				"Attempt to update which buffer is displayed on sprite, but underlying buffer {} no longer exists",
				buffer_handle
			)
		});
		sprite.image = image;
	}
}
