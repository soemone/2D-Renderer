// Modules
mod base_renderer;
mod vertex;
mod utils;
mod entity;

// Imports
use base_renderer::BaseRenderer;
use entity::EntityList;
use utils::{defaults::Float, Mat4x4, Vector};
use wgpu::core::{device, identity};
use winit::{
    dpi::PhysicalSize, 
    event_loop::EventLoop, 
    window::{
        Theme, 
        WindowBuilder
    }
};
use pollster::FutureExt as _;

fn main() {
    
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = 
        WindowBuilder::new()
            .with_title("Render Test")
            .with_theme(Some(Theme::Dark))
            .with_inner_size(PhysicalSize::new(720, 720))
            .build(&event_loop)
            .unwrap();
        
    let mut renderer = BaseRenderer::new(&window).block_on();
    let entity_list = &mut renderer.entities;
    let e1 = entity_list.add_entity();
    e1.create(|e| {
        let verts = utils::generate_regular_geometry(16, 0.1, Vector::new(0.0, 0.0), 0.0);
        let mut indices = Vec::with_capacity(verts.len());
        for i in 0..verts.len() {
            indices.push(i as u32);
        }
        let tris_right = utils::generate_triangles(indices);
        e.set_geometry(&verts, &tris_right);
    });

    e1.set_update(|e| {
        e.shear_by(0.02);
    });

    let mut x = 0;

    let mut func = |el: &mut EntityList| {
        if x == 1 { x += 2; }
    };
    renderer.set_main_loop(&mut func);

    renderer.run(event_loop);

    // drop(renderer);
}