#![no_std]

use spirv_std::glam::{Vec2, Vec3, Vec4};
use spirv_std::{spirv, Image, Sampler};

#[spirv(vertex)]
pub fn main_vs(
    position: Vec3,
    texture_coord: Vec2,
    #[spirv(position)] clip_position: &mut Vec4,
    output: &mut Vec2,
) {
    *clip_position = position.extend(1.);
    *output = texture_coord
}

#[spirv(fragment)]
pub fn main_fs(
    input: Vec2,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 1)] sampler: &Sampler,
    output: &mut Vec4,
) {
    *output = texture.sample(*sampler, input)
}
