struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

struct Color {
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> transformation_matrix: mat4x4<f32>;

// Shader argument passed into here
// Arrays don't work maybe - throws some error
// Structs work though, so I assume there's no real point trying to make them work
@group(1) @binding(1)
var<uniform> shader_args: Color;

@vertex
fn vertex(
    vertex_input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4(vertex_input.position, 1.0, 1.0) * transformation_matrix;
    output.color = shader_args.color;
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = vec4(input.color, 1.0);
    return output;
}