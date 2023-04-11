#![no_std]

use spirv_std::glam::{Vec3, Vec4};
use spirv_std::spirv;

#[spirv(vertex)]
pub fn main_vs(
    position: Vec3,
    color: Vec3,
    #[spirv(position)] clip_position: &mut Vec4,
    output: &mut Vec3,
) {
    *clip_position = position.extend(1.);
    *output = color;
}

#[spirv(fragment)]
pub fn main_fs(input: Vec3, output: &mut Vec4) {
    *output = input.extend(1.)
}
