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
	var scale = vec2<f32>(
		f32((low >> (PACKED_AXIS_SIZE * 3)) & PACKED_AXIS_MASK) + 1,
		f32((low >> (PACKED_AXIS_SIZE * 4)) & PACKED_AXIS_MASK) + 1,
	);
	
	// Unpacked quad facing dir:
	var facing = (low >> (PACKED_AXIS_SIZE * 5)) & PACKED_FACING_MASK;
	// Unpacked texture ID:
	var texture = high & PACKED_TEXTURE_MASK;
	
	var vertex_index = global_vertex_index % 4;	
	let right = f32(vertex_index == 1u || vertex_index == 2u);
	let up    = f32(vertex_index >= 2u);

    let vertex_offset = get_face(facing, voxel_offset, scale, up, right);
    let vertex_pos = vertex_offset + vec3<f32>(chunk_pos) * 32;
	
	
	
	var out: VertexOutput;
	out.position = view.clip_from_world * vec4(vertex_pos, 1.0);
	out.layer = texture;
	out.uv = vec2(right * scale.x, up * scale.y);
	
	switch (facing) {
		case 0: { // Top (y+)
			out.brightness = 1;
		}
		case 1: { // Bot (y-)
			out.brightness = 0.2;
		}
		case 2: { // Right (x+) 
			out.brightness = 0.8;
		}
		case 3: { // Left (x-)
			out.brightness = 0.6;		
		}
		case 4: { // Back (z+)
			out.brightness = 0.7;
		}
		case 5: { // Front (z-)
			out.brightness = 0.4;
		}
		default: {
			out.brightness = -1;
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

fn get_face(facing: u32, voxel_pos: vec3<f32>, scale: vec2<f32>, up: f32, right: f32) -> vec3<f32> {
    var origin: vec3<f32>;
    var right_axis: vec3<f32>;
    var up_axis: vec3<f32>;

    switch (facing) {
        case 0u: { // Top (+Y) - looking down, right=+X, up=+Z
            origin     = voxel_pos + vec3(0.0, scale.y, 0.0);
            right_axis = vec3(1.0, 0.0, 0.0);
            up_axis    = vec3(0.0, 0.0, 1.0);
        }
        case 1u: { // Bottom (-Y) - looking up, right=+X, up=+Z
			origin     = voxel_pos + vec3(scale.x, 0.0, 0.0);
			right_axis = vec3(-1.0, 0.0, 0.0);
			up_axis    = vec3(0.0, 0.0, 1.0);
        }
        case 2u: { // Right (+X) - looking left, right=+Z, up=+Y
            origin     = voxel_pos + vec3(scale.x, 0.0, 0.0);
            right_axis = vec3(0.0, 0.0, 1.0);
            up_axis    = vec3(0.0, 1.0, 0.0);
        }
        case 3u: { // Left (-X) - looking right, right=+Z, up=+Y
			origin     = voxel_pos + vec3(0.0, 0.0, scale.x);
			right_axis = vec3(0.0, 0.0, -1.0);
			up_axis    = vec3(0.0, 1.0, 0.0);
        }
        case 4u: { // Back (+Z) - looking forward, right=+X, up=+Y
            origin     = voxel_pos + vec3(scale.x, 0.0, scale.x);
            right_axis = vec3(-1.0, 0.0, 0.0);
            up_axis    = vec3(0.0, 1.0, 0.0);
        }
        case 5u: { // Front (-Z) - looking backward, right=+X, up=+Y
            origin     = voxel_pos;
            right_axis = vec3(1.0, 0.0, 0.0);
            up_axis    = vec3(0.0, 1.0, 0.0);
        }
        default: {
            origin     = voxel_pos;
            right_axis = vec3(1.0, 0.0, 0.0);
            up_axis    = vec3(0.0, 1.0, 0.0);
        }
    }
    return origin + right * right_axis * scale.x + up * up_axis * scale.y;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(array_texture, array_texture_sampler, in.uv, in.layer) * in.brightness;
}