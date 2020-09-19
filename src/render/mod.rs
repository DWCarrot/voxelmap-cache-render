pub mod data;
pub mod key;
pub mod tile;


use image::Pixel;
use image::Rgba;
use image::RgbaImage;

use crate::color::biome::Biome;
use crate::color::ColorManager;
use crate::color::BakedColorManager;
use data::TILESIZE;
use tile::Tile;


pub type GEResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct RenderOptions {
    gamma: f32,
    env_light: u8,
}

impl Default for RenderOptions {

    fn default() -> Self {
        RenderOptions {
            gamma: 1.0,
            env_light: 15,
        }
    }
}

impl RenderOptions {

    pub fn set_gamma(&mut self, gamma: f32) {
        if gamma > 0.0 {
            self.gamma = gamma;
        }
    }

    pub fn set_env_light(&mut self, light: u8) {
        self.env_light = std::cmp::min(light, 15);
    }
}


pub fn gamma_correction(c: &mut Rgba<u8>, gamma: f32) {
    let r = c[0] as f32 / 255.0;
    let g = c[1] as f32 / 255.0;
    let b = c[2] as f32 / 255.0;
    let r = r.powf(1.0 / gamma);
    let g = g.powf(1.0 / gamma);
    let b = b.powf(1.0 / gamma);
    c[0] = (r * 255.0) as u8;
    c[1] = (g * 255.0) as u8;
    c[2] = (b * 255.0) as u8;
}

pub fn light_modify(c: &mut Rgba<u8>, light: u8) {
    let r = c[0] as u16;
    let g = c[1] as u16;
    let b = c[2] as u16;
    let light = (light & 0x0F)  as u16;
    let r = r * light / 15;
    let g = g * light / 15;
    let b = b * light / 15;
    c[0] = r as u8;
    c[1] = g as u8;
    c[2] = b as u8;
}


pub fn render(tile: Tile, mgr: &BakedColorManager, options: &RenderOptions) -> RgbaImage {
    let mut panel = RgbaImage::new(TILESIZE.0, TILESIZE.1);
    let tilev = tile.view();
    for x in 0 .. TILESIZE.0 {
        for z in 0 .. TILESIZE.1 {

            let element = tilev.element(x, z);
            let biome = Biome(element.biome() as usize);
            let mut surface = {
                let layer = element.surface();
                if layer.height() > 0 {
                    let (c, props) = tile.get_color(layer.blockstate_id()); 
                    let mut c = mgr.get_modified_color(c.clone(), &props.biome_color, layer.height() as i32, &biome, props.waterlogged);
                    let light = std::cmp::max(layer.blocklight(), options.env_light);
                    light_modify(&mut c, light);
                    (c, layer.height())
                } else {
                    (Rgba::from([0, 0, 0, 0]), 0)
                }
            };
            let color = if surface.1 > 0 {
                let mut seafloor = {
                    let layer = element.seafloor();
                    if layer.height() > 0 {
                        let (c, props) = tile.get_color(layer.blockstate_id()); 
                        let mut c = mgr.get_modified_color(c.clone(), &props.biome_color, layer.height() as i32, &biome, props.waterlogged);
                        let light = std::cmp::max(layer.blocklight(), options.env_light);
                        light_modify(&mut c, light);
                        (c, layer.height())
                    } else {
                        (Rgba::from([0, 0, 0, 0]), 0)
                    }
                };
                let mut transparent = {
                    let layer = element.transparent();
                    if layer.height() > 0 {
                        let (c, props) = tile.get_color(layer.blockstate_id()); 
                        let mut c = mgr.get_modified_color(c.clone(), &props.biome_color, layer.height() as i32, &biome, props.waterlogged);
                        let light = std::cmp::max(layer.blocklight(), options.env_light);
                        light_modify(&mut c, light);
                        (c, layer.height())
                    } else {
                        (Rgba::from([0, 0, 0, 0]), 0)
                    }
                };
                let mut foliage = {
                    let layer = element.foliage();
                    if layer.height() > 0 {
                        let (c, props) = tile.get_color(layer.blockstate_id()); 
                        let mut c = mgr.get_modified_color(c.clone(), &props.biome_color, layer.height() as i32, &biome, props.waterlogged);
                        let light = std::cmp::max(layer.blocklight(), options.env_light);
                        light_modify(&mut c, light);
                        (c, layer.height())
                    } else {
                        (Rgba::from([0, 0, 0, 0]), 0)
                    }
                };
                if options.gamma != 1.0 {
                    gamma_correction(&mut surface.0, options.gamma);
                    gamma_correction(&mut seafloor.0, options.gamma);
                    gamma_correction(&mut transparent.0, options.gamma);
                    gamma_correction(&mut foliage.0, options.gamma);
                }
                let mut color = if seafloor.1 > 0 {
                    let mut color = seafloor.0;
                    if foliage.1 > 0 && foliage.1 <= surface.1 {
                        blend(&mut color, &foliage.0);
                    }
                    if transparent.1 > 0 && transparent.1 <= surface.1 {
                        blend(&mut color, &transparent.0);
                    }
                    blend(&mut color, &surface.0);
                    color
                } else {
                    surface.0
                };
                if foliage.1 > 0 && foliage.1 > surface.1 {
                    blend(&mut color, &foliage.0);
                }
                if transparent.1 > 0 && transparent.1 > surface.1 {
                    blend(&mut color, &transparent.0);
                }
                color
            } else {
                Rgba::from([0, 0, 0, 0])
            };

            panel.put_pixel(x, z, color);
        }
    }
    panel
}

fn blend<P: Pixel>(bg: &mut P, fg: &P) {
    bg.blend(fg);
}