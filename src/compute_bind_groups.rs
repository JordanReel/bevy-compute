use bevy::{prelude::*, render::render_resource::BindGroup};

#[derive(Resource)]
pub struct ComputeBindGroups(pub Vec<BindGroup>);
