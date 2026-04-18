#import bevy_pbr::mesh_view_bindings::view

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> chunk_pos: vec3<i32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1) var array_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var array_texture_sampler: sampler;

const PACKED_POS_SIZE: u32 = 5;
const PACKED_POS_MASK: u32 = (1 << PACKED_POS_SIZE) - 1; // 0b11111

const PACKED_FACING_SIZE: u32 = 3;
const PACKED_FACING_MASK: u32 = (1 << PACKED_FACING_SIZE) - 1; // 0b111

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) layer: u32,
    @location(1) uv: vec2<f32>,
    @location(2) brightness: f32,
};

@vertex
fn vertex(
	@location(0) packed_data: u32,
	@builtin(vertex_index) global_vertex_index: u32
) -> VertexOutput {
	// Unpacked voxel pos:
	var voxel_offset = vec3<f32>(
		f32(packed_data & PACKED_POS_MASK),
		f32((packed_data >> PACKED_POS_SIZE) & PACKED_POS_MASK),
		f32((packed_data >> (PACKED_POS_SIZE * 2)) & PACKED_POS_MASK),
	);
	// Unpacked quad facing dir:
	var facing = (packed_data >> (PACKED_POS_SIZE * 3)) & PACKED_FACING_MASK;
	// Unpacked texture ID:
	var texture = packed_data >> (PACKED_POS_SIZE * 3 + PACKED_FACING_SIZE);
	
	var voxel_pos = voxel_offset + vec3<f32>(chunk_pos) * 32;
	
	var vertex_index = global_vertex_index % 4;
	var vertex_offset = vec3<f32>(
		f32(vertex_index == 1 || vertex_index == 2),
		f32(facing % 2 == 0),
		f32(vertex_index >= 2),
	);
	
	var vertex_pos = voxel_pos;
	
	switch (facing) {
		case 0, 1: { // Top and Bot (y)
			vertex_pos += vertex_offset.xyz;
		}
		case 2, 3: { // Right and Left (x)
			vertex_pos += vertex_offset.yzx;
		}
		case 4, 5: { // Back and Front (z)
			vertex_pos += vertex_offset.zxy;
		}
		default: {}
	}
	
	var out: VertexOutput;
	out.position = view.clip_from_world * vec4(vertex_pos, 1.0);
	out.layer = texture;
	out.uv = vertex_offset.xz;
	
	switch (facing) {
		case 0: { // Top (y+)
			out.brightness = 1;
		}
		case 1: { // Bot (y-)
			out.brightness = 0.25;
		}
		case 2, 3: { // Right and Left (x)
			out.brightness = max(vertex_offset.z, 0.25);
		}
		case 4, 5: { // Back and Front (z)
			out.brightness = max(vertex_offset.x, 0.25);			
		}
		default: {}
	}
	
	return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(array_texture, array_texture_sampler, in.uv, in.layer) * in.brightness;
}