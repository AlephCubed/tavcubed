#import bevy_pbr::mesh_view_bindings::view

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> chunk_pos: vec3<i32>;

const PACKED_POS_SIZE: u32 = 5;
const PACKED_POS_MASK: u32 = (1 << PACKED_POS_SIZE) - 1; // 0b11111

const PACKED_FACING_SIZE: u32 = 3;
const PACKED_FACING_MASK: u32 = (1 << PACKED_FACING_SIZE) - 1; // 0b111

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
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
	
	var voxel_pos = voxel_offset + vec3<f32>(chunk_pos) * 16;
	
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
		default: {
			
		}
	}
	
	var out: VertexOutput;
	out.position = view.clip_from_world * vec4(vertex_pos, 1.0);
	
	switch (facing) {
		case 0: {
			out.color = vec4<f32>(1, 1, 1, 1);
		}
		case 1: {
			out.color = vec4<f32>(0, 0, 0, 1);
		}
		case 2: {
			out.color = vec4<f32>(1, 0, 0, 1);
		}
		case 3: {
			out.color = vec4<f32>(1, 0, 1, 1);
		}
		case 4: {
			out.color = vec4<f32>(0, 1, 0, 1);
		}
		case 5: {
			out.color = vec4<f32>(0, 0, 1, 1);
		}
		default: {
			out.color = vec4<f32>(1, 1, 0, 1);
		}
	}
	
	return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}