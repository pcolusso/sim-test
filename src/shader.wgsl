struct State {
    pos: vec2<f32>,
    dim: vec2<f32>,
    t: f32,
}

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) coord: vec2<f32>
};

@group(0)
@binding(0)
var<uniform> app_state: State;

@group(0)
@binding(1)
var<storage> gridData: array<u32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var vertices = array<vec2<f32>, 3>(
        vec2<f32>(-1., 1.),
        vec2<f32>(3.0, 1.),
        vec2<f32>(-1., -3.0),
    );
    var out: VertexOutput;
    out.coord = vertices[in.vertex_index];
    out.position = vec4<f32>(out.coord, 0.0, 1.0);

    return out;
}

fn unpack_bgra5551(packed: u32) -> vec4f {
    // Extract each channel (5 bits each for BGR, 1 bit for A)
    let b = (packed & 0x1Fu);         // bits 0-4
    let g = (packed >> 5u) & 0x1Fu;   // bits 5-9
    let r = (packed >> 10u) & 0x1Fu;  // bits 10-14
    let a = (packed >> 15u) & 0x1u;   // bit 15

    // Convert to normalized float [0,1]
    // For 5-bit channels, divide by 31 (2^5 - 1)
    // For 1-bit alpha, it's either 0 or 1
    return vec4f(
        f32(r) / 31.0,
        f32(g) / 31.0,
        f32(b) / 31.0,
        f32(a)
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = gridData[0];
    let color: vec4<f32> = unpack_bgra5551(c);
    return color;

    // // Normalize the fragment coordinates
    // let normalized_coord = in.position.xy / app_state.dim;
    // let normalized_pos = app_state.pos / app_state.dim;
    // // let normalized_coord = in.coord.xy / vec2<f32>(800.0, 600.0);

    // // Calculate the distance between the fragment and the cursor
    // let distance = distance(normalized_coord, normalized_pos);

    // let gradient = vec3<f32>(0.3 * distance, 0.3 * distance, 1.0);
    // return vec4<f32>(gradient, 1.0);
}
