use image::Primitive;
use image::Rgba;


pub fn blend(bg: &mut Rgba<u8>, fg: &Rgba<u8>) {
    let r0 = bg[0] as u32;
    let g0 = bg[1] as u32;
    let b0 = bg[2] as u32;
    let a0 = bg[3] as u32;
    let r1 = fg[0] as u32;
    let g1 = fg[1] as u32;
    let b1 = fg[2] as u32;
    let a1 = fg[3] as u32;
    let ia1 = 255 - a1;
    let r = (r0 * ia1 + r1 * a1) / 255;
    let g = (g0 * ia1 + g1 * a1) / 255;
    let b = (b0 * ia1 + b1 * a1) / 255;
    let a = std::cmp::max(a0, a1);
    bg[0] = r as u8;
    bg[1] = g as u8;
    bg[2] = b as u8;
    bg[3] = a as u8;
}