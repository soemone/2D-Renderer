struct VertexInput {
    @location(0) position: vec2<f32>,
    //@location(1) color: vec3<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> transformation_matrix: mat4x4<f32>;

@vertex
fn vertex(
    vertex_input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4(vertex_input.position, 1.0, 1.0) * transformation_matrix;
    output.color = vec3(0.05, 0.05, 0.05);//vertex_input.color;
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = input.position;//vec4(input.color, 1.0);
    return output;
}