use std::{ 
    rc::Rc, 
    slice::{ 
        IterMut, Iter 
    } 
};

use wgpu::util::DeviceExt;

use crate::{
    utils::{ 
        as_u8_slice, defaults::*, 
        Mat4x4, Vector 
    }, utils
};

pub struct EntityList {
    pub(crate) entities: Vec<Entity>,
    pub(crate) device: Rc<wgpu::Device>,
    pub(crate) queue: Rc<wgpu::Queue>,
}

impl EntityList {
    pub fn new(device: Rc<wgpu::Device>, queue: Rc<wgpu::Queue>) -> Self {
        Self { entities: vec![], device, queue }
    }

    pub fn add_entity(&mut self) -> &mut Entity {
        let entity = Entity::default(self.device.clone(), self.queue.clone());
        self.entities.push(entity);
        return self.entities.last_mut().unwrap();
    }

    pub fn get_entity(&mut self, index: usize) -> Option<&mut Entity> {
        self.entities.get_mut(index)
    }

    pub fn get_entity_unchecked(&mut self, index: usize) -> &mut Entity {
        &mut self.entities[index]
    }

    pub fn delete_entity(&mut self, index: usize) {
        self.entities.remove(index);
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Entity>{
        self.entities.iter_mut()
    }

    pub fn iter(&mut self) -> Iter<'_, Entity>{
        self.entities.iter()
    }

    pub fn count(&self) -> usize {
        self.entities.len()
    }
    
}

pub struct Entity {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) transform_buffer: wgpu::Buffer,
    pub(crate) shader_buffer: wgpu::Buffer,
    pub(crate) transform_bind_group: wgpu::BindGroup,
    pub(crate) shader_bind_group: wgpu::BindGroup,
    pub(crate) transform: Mat4x4,
    pub(crate) index_size: Index,
    pub(crate) device: Rc<wgpu::Device>,
    pub(crate) queue: Rc<wgpu::Queue>,
    pub(crate) render_pipeline: Option<wgpu::RenderPipeline>,
}

impl Entity { 

    pub fn default(device: Rc<wgpu::Device>, queue: Rc<wgpu::Queue>) -> Self {
        Self::new(&vec![], &vec![], Mat4x4::identity(), device, queue)
    }

    pub fn new(
        vertex_data: &Vec<Vector<Float>>, 
        index_data: &Vec<Index>, 
        transform: Mat4x4,
        device: Rc<wgpu::Device>, 
        queue: Rc<wgpu::Queue>,
    ) -> Self {
        let (vertex_slice, index_slice) = (vertex_data.as_slice(), index_data.as_slice());
         
        let transform_buffer = Self::transform_buffer(&device, transform);
        let shader_buffer = Self::shader_args_buffer::<[f32; 1]>(&device, [0.0]);
        
        let transform_layout = utils::generate_transform_layout(&device);
        let shader_layout = utils::generate_shader_args_layout(&device);

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Position Bind Group"), 
            layout: &transform_layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }]
        });

        
        let shader_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Shader Arguments Bind Group"), 
            layout: &shader_layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: shader_buffer.as_entire_binding(),
            }]
        });


        Entity {
            index_size: index_data.len() as Index,
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: as_u8_slice(vertex_slice),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: as_u8_slice(index_slice),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }),
            transform_buffer,
            shader_buffer,
            transform_bind_group,
            shader_bind_group,
            render_pipeline: None,
            transform,
            device,
            queue,
        }
    }

    pub fn set_geometry(&mut self, vertices: &Vec<Vector<Float>>, indices: &Vec<Index>) {
        self.index_size = indices.len() as Index;

        self.index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: as_u8_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        self.vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: as_u8_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
    }

    fn send_transform(&mut self, transform: Mat4x4) {
        self.queue.write_buffer(&self.transform_buffer, 0, as_u8_slice(&[transform]));
    }

    pub fn translate_by(&mut self, displacement: Vector<Float>) {
        self.transform.translate_by(displacement);
        self.send_transform(self.transform);
    }

    pub fn translate_to(&mut self, position: Vector<Float>) {
        self.transform.translate_to(position);
        self.send_transform(self.transform);
    }

    pub fn rotate_to(&mut self, angle: Float) {
        self.transform.rotate_to(angle);
        self.send_transform(self.transform);
    }

    pub fn rotate_by(&mut self, displacement_angle: Float) {
        self.transform.rotate_by(displacement_angle);
        self.send_transform(self.transform);
    }

    pub fn scale_to(&mut self, scale: Vector<Float>) {
        self.transform.scale_to(scale);
        self.send_transform(self.transform);
    }

    pub fn scale_by(&mut self, scale_factor: Vector<Float>) {
        self.transform.scale_by(scale_factor);
        self.send_transform(self.transform);
    }

    pub fn shear_to(&mut self, shear_angle: Vector<Float>) {
        self.transform.shear_to(shear_angle);
        self.send_transform(self.transform);
    }

    pub fn shear_by(&mut self, shear_displacement_angle: Vector<Float>) {
        self.transform.shear_by(shear_displacement_angle);
        self.send_transform(self.transform);
    }

    pub fn angle(&self) -> Float { self.transform.angle() }

    pub fn position(&self) -> Vector<Float> { self.transform.position() }
    
    pub fn scale(&self) -> Vector<Float> { self.transform.scale() }

    pub fn get_transform(&self) -> Mat4x4 { self.transform }
    
    pub fn set_transform(&mut self, new_transform: Mat4x4) { self.transform = new_transform; }

    pub fn set_shader(&mut self, shader: wgpu::ShaderModuleDescriptor) {
        self.render_pipeline = 
            Some(
                utils::generate_render_pipeline(
                    &self.device, 
                    wgpu::TextureFormat::Rgba8UnormSrgb, 
                    self.device.create_shader_module(shader)
                )
            );
    }

    pub fn set_shader_args<T>(&mut self, args: T) {
        let shader_buffer = Self::shader_args_buffer(&self.device, args);
        let shader_layout = utils::generate_shader_args_layout(&self.device);

        self.shader_buffer = shader_buffer;
        
        self.shader_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Shader Arguments Bind Group"), 
            layout: &shader_layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: self.shader_buffer.as_entire_binding(),
            }]
        });
    }

    /// Use this when updating shader data per frame with the SAME TYPE of data (Size of data has to be the same as the last instance sent to `set_shader_args`) 
    pub fn send_shader_args<T>(&mut self, args: T) {
        self.queue.write_buffer(&self.shader_buffer, 0, as_u8_slice(&[args]));
    }

    fn transform_buffer(device: &wgpu::Device, transform: Mat4x4) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Position Buffer"),
            contents: as_u8_slice(&[transform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn shader_args_buffer<T>(device: &wgpu::Device, arguments: T) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Arguments Buffer"),
            contents: as_u8_slice(&[arguments]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

}