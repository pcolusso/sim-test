struct State {
    pos: vec2<f32>, // cursor position
    dim: vec2<f32>, // window dimensions
    t: f32,         // time
    grid_dimensions: vec2<u32>
}

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) coord: vec2<f32>
};

@group(0)
@binding(0)
var<uniform> app_state: State;

@group(0)
@binding(1)
var<storage> grid_data: array<u32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var vertices = array<vec2<f32>, 3>(
        vec2<f32>(-1., 1.),
        vec2<f32>(3.0, 1.),
        vec2<f32>(-1., -3.0),
    );
    var out: VertexOutput;
    out.coord = vertices[in.vertex_index];
    out.pos = vec4<f32>(out.coord, 0.0, 1.0);

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
    let total_width = 1000.0;
    let cell_size = total_width / f32(app_state.grid_dimensions.x) * 0.9;
    let border_size = total_width / f32(app_state.grid_dimensions.x) * 0.1;

    let grid_width = 100.0; // TODO: can we instead derive this from the buf, assuming it's square, or via uniform?

    let t_w = (grid_width * cell_size) + (grid_width * border_size + 1.0); // +1?

    // ATTEMPT: being slick
    //let total_dimensions = vec2(f32(app_state.grid_dimensions.x), f32(app_state.grid_dimensions.y)); // square for now.
    let total_dimensions = vec2(t_w, t_w);
    // We want to center the grid, so we calculate where coords should start.
    let offset = (app_state.dim - total_dimensions) / 2.0;
    // Translate to grid space
    let grid_pos = in.pos.xy - offset;
    let outside = grid_pos < vec2<f32>(0.0) || grid_pos >= total_dimensions;

    // render background
    if (any(outside)) { // THIS IS COOL
        return vec4(0.14, 0.2, 0.52, 1.0);
    }

    let cell_and_border = cell_size + border_size;
    // Get x/y of each cell.
    let grid_coord = floor(grid_pos / cell_and_border);
    // Translate to cell space
    let local = grid_pos - (grid_coord * cell_and_border);
    let is_border = local < vec2(border_size, border_size);

    if (any(is_border)) {
        return vec4(1.0, 1.0, 1.0, 0.1);
    }

    let grid_idx = u32(grid_coord.y * 50 + grid_coord.x);
    let value = grid_data[grid_idx];

    return unpack_bgra5551(value);
}
