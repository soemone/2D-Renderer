use std::{borrow::Borrow, rc::Rc};

use wgpu::{core::device::queue, util::DeviceExt};

use crate::{utils::{ as_u8_slice, defaults::*, Mat4x4, Vector }, vertex::Vertex};

pub struct EntityList {
    pub(crate) entities: Vec<Entity>,
    pub(crate) device: Rc<wgpu::Device>,
    pub(crate) queue: Rc<wgpu::Queue>,
}

impl EntityList {
    pub fn new(device: Rc<wgpu::Device>, queue: Rc<wgpu::Queue>) -> Self {
        Self { entities: vec![], device, queue }
    }

    // pub fn add(&mut self, entity: Entity) { self.entities.push(entity) }

    pub fn add_entity(&mut self) -> &mut Entity {
        let entity = Entity::default(self.device.clone(), self.queue.clone());
        self.entities.push(entity);
        return self.entities.last_mut().unwrap();
    }

    pub fn first_run(&mut self) {
        for mut entity in &mut self.entities {
            (entity.create_fn)(&mut entity);
        }
    }

    pub fn update(&mut self) {
        for mut entity in &mut self.entities {
            (entity.update_fn)(&mut entity);
        }
    }

    // pub fn generate_entity(&mut self, vertex_data: &Vec<Vector<Float>>, index_data: &Vec<u32>) -> usize {
    //     let entity = Entity::new(vertex_data, index_data, Mat4x4::identity(), &self.device, &self);
    //     self.entities.push(entity);
    //     return self.entities.len() - 1;
    // }

    // pub fn get_entity(&mut self, index: usize) -> Option<&mut Entity> {
    //     self.entities.get_mut(index)
    // }
}

pub struct Entity {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) transform_buffer: wgpu::Buffer,
    pub(crate) transform_bind_group: wgpu::BindGroup,
    pub(crate) transform: Mat4x4,
    pub(crate) index_size: Index,
    pub(crate) update_fn: fn(&mut Entity) -> (),
    pub(crate) create_fn: fn(&mut Entity) -> (),
    pub(crate) device: Rc<wgpu::Device>,
    pub(crate) queue: Rc<wgpu::Queue>,
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
        queue: Rc<wgpu::Queue>
    ) -> Self {
        let (vertex_slice, index_slice) = (vertex_data.as_slice(), index_data.as_slice());
         
        let transform_buffer = Self::transform_buffer(&device, transform);
        
        let transform_layout = Self::transform_layout(&device);

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Position Bind Group"), 
            layout: &transform_layout, 
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }]
        });
        
        fn update_fn(_a: &mut Entity) {}
        fn create_fn(_a: &mut Entity) {}

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
            transform_bind_group,
            transform,
            update_fn,
            create_fn,
            device,
            queue,
        }
    }

    pub fn create(&mut self, function: fn(&mut Entity) -> ()) {
        self.create_fn = function;
    }

    pub fn set_update(&mut self, function: fn(&mut Entity) -> ()) { 
        self.update_fn = function;
    }

    pub fn set_geometry(&mut self, vertices: &Vec<Vector<Float>>, indices: &Vec<Index>) {
        // Hi?
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

        // queue.write_buffer(&self.vertex_buffer, 0, as_u8_slice(vertices));
        // queue.write_buffer(&self.index_buffer, 0, as_u8_slice(indices));
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

    pub fn shear_to(&mut self, shear_angle: Float) {
        self.transform.shear_to(shear_angle);
        self.send_transform(self.transform);
    }

    pub fn shear_by(&mut self, shear_displacement_angle: Float) {
        self.transform.shear_by(shear_displacement_angle);
        self.send_transform(self.transform);
    }

    pub fn angle(&self) -> Float { self.transform.angle() }

    pub fn position(&self) -> Vector<Float> { self.transform.position() }
    
    pub fn scale(&self) -> Vector<Float> { self.transform.scale() }

    pub fn get_transform(&self) -> Mat4x4 { self.transform }
    
    pub fn set_transform(&mut self, new_transform: Mat4x4) { self.transform = new_transform; }

    fn transform_buffer(device: &wgpu::Device, transform: Mat4x4) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Entity Position Buffer"),
            contents: as_u8_slice(&[transform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn transform_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Transform Bind Group Layout Desc"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

}