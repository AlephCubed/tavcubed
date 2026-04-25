#import bevy_pbr::mesh_view_bindings::view

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> chunk_pos: vec3<i32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(1) var array_texture: texture_2d_array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var array_texture_sampler: sampler;

const PACKED_AXIS_SIZE: u32 = 5;
const PACKED_AXIS_MASK: u32 = (1 << PACKED_AXIS_SIZE) - 1; // 0b11111

const PACKED_FACING_SIZE: u32 = 3;
const PACKED_FACING_MASK: u32 = (1 << PACKED_FACING_SIZE) - 1; // 0b111

const PACKED_TEXTURE_SIZE: u32 = 16;
const PACKED_TEXTURE_MASK: u32 = (1 << PACKED_TEXTURE_SIZE) - 1; // 0xFFFF

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) layer: u32,
    @location(1) uv: vec2<f32>,
    @location(2) brightness: f32,
};

@vertex
fn vertex(
	@location(0) packed_data: vec2<u32>,
	@builtin(vertex_index) global_vertex_index: u32
) -> VertexOutput {
	let low = packed_data.x;
	let high = packed_data.y;
	// Unpacked voxel pos:
	var voxel_offset = vec3<f32>(
		f32(low & PACKED_AXIS_MASK),
		f32((low >> PACKED_AXIS_SIZE) & PACKED_AXIS_MASK),
		f32((low >> (PACKED_AXIS_SIZE * 2)) & PACKED_AXIS_MASK),
	);
	var size = vec2<f32>(
		f32((low >> (PACKED_AXIS_SIZE * 3)) & PACKED_AXIS_MASK),
		f32((low >> (PACKED_AXIS_SIZE * 4)) & PACKED_AXIS_MASK),
	);
	
	// Unpacked quad facing dir:
	var facing = (low >> (PACKED_AXIS_SIZE * 5)) & PACKED_FACING_MASK;
	// Unpacked texture ID:
	var texture = high & PACKED_TEXTURE_MASK;
	
	var voxel_pos = voxel_offset + vec3<f32>(chunk_pos) * 32;
	var vertex_index = global_vertex_index % 4;
	
	
	
	let basis = get_face_basis(facing);
    
    let u = f32(vertex_index == 1 || vertex_index == 2) * (size.x + 1);
    let v = f32(vertex_index >= 2) * (size.y + 1);
    
    let vertex_offset = basis.origin + (u * basis.u_axis) + (v * basis.v_axis);
    let vertex_pos = voxel_pos + vertex_offset;
	
	
	
	var out: VertexOutput;
	out.position = view.clip_from_world * vec4(vertex_pos, 1.0);
	out.layer = texture;
    out.uv = vec2<f32>(u, v);
	
	switch (facing) {
		case 0: { // Top (y+)
			out.brightness = 1;
		}
		case 1: { // Bot (y-)
			out.brightness = 0.25;
		}
		default: {
			out.brightness = max(vertex_offset.y, 0.25);
		}
	}
	
	return out;
}

struct FaceBasis {
	/// Bottom-left corner of the face
    origin: vec3<f32>, 
    /// Direction of increasing U (and X on the quad)
    u_axis: vec3<f32>,
    /// direction of increasing V (and Y on the quad)
    v_axis: vec3<f32>, 
}

fn get_face_basis(facing: u32) -> FaceBasis {
    switch (facing) {
        case 0: { // Top (+Y)
            return FaceBasis(vec3(0,1,0), vec3(1,0,0), vec3(0,0,1));
        }
        case 1: { // Bottom (-Y)
            return FaceBasis(vec3(0,0,1), vec3(1,0,0), vec3(0,0,-1));
        }
        case 2: { // Right (+X)
            return FaceBasis(vec3(1,1,1), vec3(0,0,-1), vec3(0,-1,0));
        }
        case 3: { // Left (-X)
            return FaceBasis(vec3(0,1,0), vec3(0,0,1), vec3(0,-1,0));
        }
        case 4: { // Back (+Z)
            return FaceBasis(vec3(0,1,1), vec3(1,0,0), vec3(0,-1,0));
        }
        case 5: { // Front (-Z)
            return FaceBasis(vec3(1,1,0), vec3(-1,0,0), vec3(0,-1,0));
        }
        default: {
            return FaceBasis(vec3(0,0,0), vec3(1,0,0), vec3(0,1,0));
        }
    }
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(array_texture, array_texture_sampler, in.uv, in.layer) * in.brightness;
}