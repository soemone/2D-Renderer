// Modules
mod base_renderer;
mod vertex;
mod utils;
mod entity;
mod circle;

// Imports
use base_renderer::BaseRenderer;
use entity::EntityList;
use utils::Vector;
use circle::Circle;
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
            .with_inner_size(PhysicalSize::new(1200, 1200))
            .build(&event_loop)
            .unwrap();
        
    let mut renderer = BaseRenderer::new(&window).block_on();
    let entity_list = &mut renderer.entities;

    let mut circles = Vec::new();

    circles.push(
        Circle::new(
            entity_list,
            1, 
            0.02, 
            Vector::new(0.0, 0.5), 
            Vector::new(-0.0002, 0.0), 
            Vector::new(0.0, 0.0)
        )
    );

    circles.push(
        Circle::new(
            entity_list,
            1, 
            0.02, 
            Vector::new(-0.4330127019, -0.25), 
            Vector::new(0.0001, -0.00017320508076), 
            Vector::new(0.0, 0.0)
        )
    );

    circles.push(
        Circle::new(
            entity_list, 
            1, 
            0.02, 
            Vector::new(0.4330127019, -0.25), 
            Vector::new(0.0001, 0.0001732), 
            Vector::new(0.0, 0.0)
        )
    );

    let func = |el: &mut EntityList| {
        // Apply gravity on each circle
        Circle::gravity(&mut circles);
        // Update each circle's data
        Circle::update(&mut circles, el);
    };

    renderer.set_main_loop(func);

    renderer.run(event_loop);
    // drop(renderer);
}