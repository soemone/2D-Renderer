pub mod defaults {
    pub type Float = f32;
    pub type UInt = usize;
    pub type Index = u32;
    pub const PI: Float = std::f32::consts::PI;
}

use defaults::{
    Float, UInt, Index
};

pub fn as_u8_slice<T>(p: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            (p as *const [T]) as *const u8, 
            core::mem::size_of_val(p)
        )
    }
}    

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Mat4x4 {
    value: [[Float; 4]; 4],
}

impl Mat4x4 {
    pub fn new(
         _0: Float,  _1: Float,  _2: Float,  _3: Float, 
         _4: Float,  _5: Float,  _6: Float,  _7: Float, 
         _8: Float,  _9: Float, _10: Float, _11: Float,
        _12: Float, _13: Float, _14: Float, _15: Float,
    ) -> Self {
        Self {
            value: [
                [ _0,  _1,  _2,  _3],
                [ _4,  _5,  _6,  _7],
                [ _8,  _9, _10, _11],
                [_12, _13, _14, _15]
            ]
        }
    }

    pub fn identity() -> Self {
        Self::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn translate_by(&mut self, displacement_vec: Vector<Float>) {
        self.value[0][2] += displacement_vec.x();
        self.value[1][2] += displacement_vec.y();
    }

    pub fn translate_to(&mut self, position_vec: Vector<Float>) {
        self.value[0][2] = position_vec.x();
        self.value[1][2] = position_vec.y();
    }

    pub fn position(&self) -> Vector<Float> {
        Vector::new(self.value[0][2], self.value[1][2])
    }

    pub fn rotate_by(&mut self, displacement_angle: Float) {
        let initial_angle = Float::atan2(self.value[1][0], self.value[0][0]);
        let (cval, sval) = (Float::cos(initial_angle + displacement_angle), Float::sin(initial_angle + displacement_angle));
        self.value[0][0] = cval;
        self.value[1][1] = cval;
        self.value[0][1] = -sval;
        self.value[1][0] = sval;
    }

    pub fn rotate_to(&mut self, angle: Float) {
        let (cval, sval) = (Float::cos(angle), Float::sin(angle));
        self.value[0][0] = cval;
        self.value[1][1] = cval;
        self.value[0][1] = -sval;
        self.value[1][0] = sval;
    }

    pub fn angle(&self) -> Float {
        Float::atan2(self.value[1][0], self.value[0][0])
    }

    pub fn scale_to(&mut self, scale: Vector<Float>) {
        self.value[0][0] = scale.x();
        self.value[1][1] = scale.y();
    }

    pub fn scale_by(&mut self, scale_factor: Vector<Float>) {
        self.value[0][0] *= scale_factor.x();
        self.value[1][1] *= scale_factor.y();
    }

    pub fn scale(&self) -> Vector<Float> {
        Vector::new(self.value[0][0], self.value[1][1])
    }

    // TODO: Is this the correct way to shear?
    pub fn shear_to(&mut self, shear_angle: Float) {
        self.value[0][1] = shear_angle;
        // self.value[1][0] = shear_angle;
    }

    pub fn shear_by(&mut self, shear_displacement_angle: Float) {
        self.value[0][1] += shear_displacement_angle;
        // self.value[1][0] += shear_displacement_angle;
    }
}

#[derive(Clone, Debug)]
pub struct Polygon {
    vertices: Vec<Vector<Float>>,
    indices: Vec<UInt>,
}

impl Polygon {
    pub fn new(vertices: Vec<Vector<Float>>, indices: Vec<UInt>) -> Self {
        Polygon { vertices, indices }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Vector<T: Clone + Copy> {
    pub pos: [T; 2],
}

impl<T: Copy + Clone> Vector<T> {
    pub fn new(x: T, y: T) -> Vector<T> {
        Vector { pos: [x, y] }
    }

    pub fn x(&self) -> T { self.pos[0] }
    pub fn y(&self) -> T { self.pos[1] }

    pub fn set_x(&mut self, value: T) { self.pos[0] = value; }
    pub fn set_y(&mut self, value: T) { self.pos[1] = value; }
}

impl std::fmt::Display for Vector<Float> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y:{})", self.pos[0], self.pos[1])
    }
}

fn internal_gen_arc(sides: u16, radius: Float, center: Vector<Float>, phase: Float, angle: Float) -> Vec<Vector<Float>> {
    let mut points = Vec::with_capacity(sides as usize);
    let step = 2.0 * defaults::PI / sides as Float;
    for i in 0..sides {
        let mut step_angle = step * i as Float;
        let mut done = false;
        if step_angle >= angle {
            step_angle = angle;
            done = true;
        }

        let pos = 
            Vector::new(
                center.x() + Float::sin(phase + step_angle) * radius, 
                center.y() + Float::cos(phase + step_angle) * radius
            );
        points.push(pos);
        if done { break; }
    }
    return points;
}

pub fn generate_regular_geometry(sides: u16, radius: Float, center: Vector<Float>, phase: Float) -> Vec<Vector<Float>> {
    internal_gen_arc(sides, radius, center, phase, defaults::PI * 2.0)
}

pub fn generate_arc(sides: u16, radius: Float, center: Vector<Float>, phase: Float, angle: Float) -> Vec<Vector<Float>> {
    let mut arc = internal_gen_arc(sides, radius, center, phase, angle);
    arc.insert(0, Vector::new(0.0, 0.0));
    arc
}

// Generate a triangle set that uses the least possible triangles to fill a given set of points - probably
pub fn generate_triangles(points: Vec<u32>) -> Vec<u32> {
    if points.len() < 3 { return Vec::new(); }
    let num_triangles = ((points.len() as Float + 0.5) / 2.0) as u16;
    let mut tris = Vec::with_capacity(points.len() - 2 as usize);
    let mut i = 0;
    let mut new_compute_points = Vec::with_capacity((points.len() + 1) / 2);
    loop {
        let pt = points[i];
        let next_pt = points[i + 1];
        let end_pt = if i + 2 >= points.len() { points[0] } else { points[i + 2] };
        let tri = [end_pt, next_pt, pt];
        tris.extend_from_slice(&tri);
        new_compute_points.push(pt);
        i += 2;
        if (i / 2) as u16 >= num_triangles { break;}
    }
    // Push the last point for odd sided polygons
    if points.len() & 1 != 0  {
        new_compute_points.push(points[points.len() - 1]);
    }
    let res = generate_triangles(new_compute_points);
    tris.extend_from_slice(res.as_slice());
    return tris;
}


pub fn triangulate_polygon() {

}