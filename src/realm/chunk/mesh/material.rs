use crate::realm::block::data::load::load_core_blocks;
use crate::realm::block::data::registry::BlockRegistry;
use crate::realm::chunk::ChunkPlugin;
use crate::realm::chunk::mesh::ChunkMesh;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use bevy_auto_plugin::prelude::{AutoPlugin, auto_add_plugin, auto_plugin, auto_system};

#[derive(AutoPlugin)]
#[auto_add_plugin(plugin = ChunkPlugin)]
pub struct ChunkMaterialPlugin;

impl Plugin for ChunkMaterialPlugin {
    #[auto_plugin]
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bindless]
pub struct ChunkMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub texture_array: Handle<Image>,
}

impl ChunkMaterial {
    const SHADER_ASSET_PATH: &str = "shaders/chunk.wgsl";
}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        Self::SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        Self::SHADER_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[ChunkMesh::ATTRIBUTE_PACKED_DATA.at_shader_location(0)])?;

        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

#[derive(Resource)]
pub struct ChunkMaterialHandle(pub Handle<ChunkMaterial>);

#[auto_system(plugin = ChunkMaterialPlugin, schedule = Startup, config(after = load_core_blocks))]
fn load_chunk_material(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
) {
    let material = materials.add(ChunkMaterial {
        texture_array: registry.textures.clone().unwrap(),
    });

    debug!("Created chunk material");

    commands.insert_resource(ChunkMaterialHandle(material))
}
