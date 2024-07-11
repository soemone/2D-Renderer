use crate::utils::{ 
    defaults::*, 
    Mat4x4, 
    Vector 
};
use crate::utils;
use crate::entity::EntityList;

// required for wgpu to not scream at you
#[repr(align(16))]
struct CircleColor {
    color: [f32; 3],
}

pub struct Circle {
    mass: usize,
    radius: f32,
    index: usize,
    position: Vector<Float>,
    velocity: Vector<Float>,
    acceleration: Vector<Float>,
}

impl Circle {
    pub fn new(entity_list: &mut EntityList, mass: usize, radius: f32, position: Vector<Float>, velocity: Vector<Float>, acceleration: Vector<Float>) -> Circle {
        let index = entity_list.count();
        let circle = entity_list.add_entity();
        
        let verts = utils::generate_regular_geometry(20, radius, Vector::new(0.0, 0.0), 0.0);
        let mut indices = Vec::with_capacity(verts.len());
        for i in 0..verts.len() {
            indices.push(i as u32);
        }
        let tris_right = utils::generate_triangles(indices);
        circle.set_geometry(&verts, &tris_right);

        let mut circle_transform = Mat4x4::identity();
        circle_transform.translate_to(position);

        circle.set_transform(circle_transform);

        if index != 1 {
            circle.set_shader(wgpu::include_wgsl!("color_shader.wgsl"));

            let circle_color = CircleColor { color: [0.0, 0.5, 0.5] };
    
            circle.set_shader_args(circle_color);
        }

        // Test send_shader_args fn
        if index == 2 {
            let circle_color = CircleColor { color: [0.6, 0.4, 0.1] };
            circle.send_shader_args(circle_color);
        }

        Circle {
            mass,
            radius,
            position,
            velocity,
            acceleration,
            index
        }
    }

    // Update gravity calculations for each circle within a provided vec containing circles
    pub fn gravity(circles: &mut Vec<Circle>) {
        let circle_len = circles.len();
        for i in 0..circle_len {
            let circle_pos = circles[i].position;
            for j in 0..circle_len {
                if i == j { continue; }

                const G: f32 = 0.0000001;

                let other_circle_pos = circles[j].position;
                let other_circle_mass = circles[j].mass as f32;
                let pos_diff = 
                    Vector::new(
                        circle_pos.x() - other_circle_pos.x(), 
                        circle_pos.y() - other_circle_pos.y()
                    );

                let pos_diff_mag_sq = pos_diff.x() * pos_diff.x() + pos_diff.y() * pos_diff.y();
                let force_mag = G * other_circle_mass / pos_diff_mag_sq;
                
                let force = Vector::new(
                    -(pos_diff.x() / pos_diff_mag_sq.sqrt()) * force_mag, 
                    -(pos_diff.y() / pos_diff_mag_sq.sqrt()) * force_mag
                );

                let acceleration_circle = force;

                unsafe {
                    circles.get_unchecked_mut(i).acceleration.add_vec(acceleration_circle);
                }
            }
        }
    }

    // Update circle data in each circle element within a given vec of circles
    pub fn update(circles: &mut Vec<Circle>, entity_list: &mut EntityList) {
        for circle in circles {
            let entity = entity_list.get_entity_unchecked(circle.index);
            
            circle.velocity.add_vec(circle.acceleration);

            circle.position.add_vec(circle.velocity);
            entity.translate_by(circle.velocity);

            circle.acceleration = Vector::new(0.0, 0.0);
        }
    }
}
