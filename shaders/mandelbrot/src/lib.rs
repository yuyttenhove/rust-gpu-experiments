#![no_std]

use core::ops::{Add, Mul};

use spirv_std::{
    glam::{vec2, vec3, vec4, Vec4},
    spirv,
};

#[derive(Clone, Copy)]
struct C32 {
    r: f32,
    i: f32,
}

impl C32 {
    fn new(r: f32, i: f32) -> Self { Self { r, i } }

    const ZERO: Self = Self { r: 0., i: 0. };

    fn norm2(&self) -> f32 {
        self.r * self.r + self.i * self.i
    }
}

impl Mul<C32> for C32 {
    type Output = C32;

    fn mul(self, rhs: C32) -> Self::Output {
        Self {
            r: self.r * rhs.r - self.i * rhs.i,
            i: self.r * rhs.i + self.i * rhs.r,
        }
    }
}

impl Add<C32> for C32 {
    type Output = C32;

    fn add(self, rhs: C32) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            i: self.i + rhs.i,
        }
    }
}

fn mandelbrot_iter(z0: C32, max_iter: usize) -> f32 {
    let mut z = C32::ZERO;
    let mut iter = 0;

    while z.norm2() < 4. && iter < max_iter {
        z = z * z + z0;
        iter += 1;
    }

    if iter < max_iter {
        iter as f32 / max_iter as f32
    } else {
        0.
    }
}

#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] index: i32, #[spirv(position)] clip_position: &mut Vec4) {
    // Generate screen filling triangle
    let x = (index - 1) as f32 * 3.;
    let y = ((index % 2) as f32 * 2. - 1.) * 3.;
    *clip_position = vec4(x, y, 0., 1.);
}

#[spirv(fragment)]
pub fn main_fs(#[spirv(frag_coord)] coordinates: Vec4, frag_color: &mut Vec4) {
    let texel = vec2(coordinates.x / 800., coordinates.y / 800.);

    let v = mandelbrot_iter(C32::new(3. * texel.x - 2., 3. * texel.y - 1.5), 250);

    *frag_color = vec3(v, v, v).extend(1.);
}
