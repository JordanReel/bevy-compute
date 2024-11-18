use bevy::prelude::*;

use crate::{shader_buffer_set::ShaderBufferSet, DoubleBufferedSprite};

pub fn swap_sprite_buffers(
	mut sprite: Query<(&mut Handle<Image>, &DoubleBufferedSprite)>, buffer_set: ResMut<ShaderBufferSet>,
) {
	for (mut sprite, DoubleBufferedSprite(buffer_handle)) in sprite.iter_mut() {
		let image = buffer_set.image_handle(*buffer_handle).unwrap();
		if *sprite != image {
			*sprite = image;
		}
	}
}
