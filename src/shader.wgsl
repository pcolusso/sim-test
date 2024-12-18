struct State {
    pos: vec2<f32>,
    dim: vec2<f32>,
    t: f32
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize the fragment coordinates
    let normalized_coord = in.position.xy / app_state.dim;
    let normalized_pos = app_state.pos / app_state.dim;
    // let normalized_coord = in.coord.xy / vec2<f32>(800.0, 600.0);

    // Calculate the distance between the fragment and the cursor
    let distance = distance(normalized_coord, normalized_pos);

    let gradient = vec3<f32>(0.3 * distance, 0.3 * distance, 1.0);
    return vec4<f32>(gradient, 1.0);
}
