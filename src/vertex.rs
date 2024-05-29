use wgpu::{
    vertex_attr_array, 
    VertexAttribute
};

use crate::utils::Vector;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: [f32; 3],
}

impl Vertex {

    // Constants
    const ATTRIBUTES: [VertexAttribute; 1] = vertex_attr_array![0 => Float32x2];
    //, 1 => Float32x3];

    // Functions
    pub fn new(filepath: String) -> (Vec<Vertex>, Vec<u32>) {
        println!("Reading vertex data file...");
        let file_contents = std::fs::read_to_string(filepath).expect("Failed to read file");
        let mut vertex_buffer = Vec::new(); 
        println!("Opened vertex data file.");
        let time = std::time::Instant::now();
        let mut is_index_buffer = false;
        let mut index_buffer = Vec::new();
        // TODO: Is splitting and trimming faster, or is replacing \r\n and \r with \n faster?
        // From unscientific tests, the latter is faster
        file_contents
            .replace("\r\n", "\n")
            .replace("\r", "\n")
            .split("\n")
            // .split(&['\n', '\r'][..])
            .for_each(|data: &str| {
                // let data = data.trim();
                // if data == "" { return; }

                // Deal with index buffer contents
                if data.trim() == "~" {
                    is_index_buffer = true;
                    return;
                }

                if is_index_buffer {
                    data
                        .split(" ")
                        .for_each(|elem| {
                            let index = elem.parse::<u32>().expect("Failed to parse index buffer!");
                            index_buffer.push(index)
                        });
                    return;
                }

                // [v,v,v c,c,c]
                // [v,v,v] can also be parsed with the color being white
                let mut data_split = data.split(" ");
                // vectors first
                let position_data = data_split.next().expect("Failed to parse file. Expected position data");
                let color_data = data_split.next().unwrap_or("1.0,1.0,1.0");
                // Parse positions
                let mut position = [0.0; 3];
                let mut i = 0;
                position_data
                    .split(",")
                    .for_each(|num: &str| {
                        let parsed = num.parse::<f32>();
                        position[i] = parsed.expect("Failed to parse position number");
                        i += 1;
                    });
                // Parse colors
                let mut color = [1.0; 3];
                i = 0;
                color_data
                    .split(",")
                    .for_each(|num: &str| {
                        let parsed = num.parse::<f32>();
                        color[i] = parsed.unwrap_or(1.0);
                        i += 1;
                    });
                // Add to the vertex list
                vertex_buffer.push(Vertex { position, color })
            });
        println!("Finished parsing vertex file in {:?}", std::time::Instant::now() - time);
        (vertex_buffer, index_buffer)
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vector<f32>>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

}
