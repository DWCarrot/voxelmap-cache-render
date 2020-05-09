pub mod blockstate;
pub mod biome;
pub mod de;
pub mod calculate;

use std::iter::Empty;
use std::collections::hash_map::HashMap;

use image::Rgb;
use image::Rgba;
use image::Pixel;
use image::GrayImage;
use image::RgbaImage;

use biome::Biome;
use biome::BiomeColor;
use biome::InnerColor;
use de::BlockStateC;

pub trait ColorManager {

    fn get_basic_color<'a, I: Iterator<Item=&'a str>>(&'a self, block: &'a str, key: I, water_logged: bool) -> Rgba<u8>;
    
    fn get_modified_color(&self, basic: Rgba<u8>, inner_color: &InnerColor, height: i32, biome: &Biome, water_logged: bool) -> Rgba<u8>;

}


pub struct BakedColorManager {

    index: HashMap<String, BlockStateC>,

    colormap: Vec<Rgba<u8>>,

    weightmap: Vec<u8>,

    biome_color: BiomeColor,

    water_basic: (Rgba<u8>, u16),

}

impl BakedColorManager {

    pub fn from_raw(index: HashMap<String, BlockStateC>, colormap: RgbaImage, weightmap: GrayImage, biome_color: BiomeColor) -> Self {
        let mut obj = BakedColorManager {
            index,
            colormap: colormap.pixels().map(Clone::clone).collect(),
            weightmap: weightmap.into_raw(),
            biome_color,
            water_basic: (Rgba::from([0, 0, 0, 0]), 0),
        };
        if let Some(blockstate) = obj.index.get("minecraft:water") {
            let it: Empty<&str> = Empty::default();
            if let Some(index) = blockstate.get(it).first() {
                let water_index = index.clone();
                obj.water_basic = (obj.colormap[water_index], obj.weightmap[water_index] as u16);
            }
        }       
        obj
    }
}

impl ColorManager for BakedColorManager {

    fn get_basic_color<'a, I: Iterator<Item=&'a str>>(&'a self, block: &'a str, key: I, water_logged: bool) -> Rgba<u8> {
        if let Some(blockstate) = self.index.get(block) {
            let colors_index = blockstate.get(key);
            if colors_index.len() == 0 && !water_logged {
                return Rgba::from([0, 0, 0, 0]);
            }
            let mut colors_tuple: Vec<_> = colors_index.into_iter().map(|i| (self.colormap[i], self.weightmap[i] as u16)).collect();
            colors_tuple.sort_by_key(|t| t.1);
            let max_w = if water_logged {
                255
            } else {
                colors_tuple.last().unwrap().1
            };
            if max_w == 0 {
                return Rgba::from([0, 0, 0, 0]);
            }
            let mut final_color = Rgba::from([0, 0, 0, 0]);
            for (mut color, w) in colors_tuple {
                if w * 4 < max_w {
                    continue;
                }
                color[0] = (color[0] as u16 * w / max_w) as u8;
                color[1] = (color[1] as u16 * w / max_w) as u8;
                color[2] = (color[2] as u16 * w / max_w) as u8;
                color[3] = (color[3] as u16 * w / max_w) as u8;
                final_color.blend(&color)
            }
            final_color
        } else {
            Rgba::from([0, 0, 0, 0])
        }
    }
    
    fn get_modified_color(&self, mut basic: Rgba<u8>, inner_color: &InnerColor, height: i32, biome: &Biome, water_logged: bool) -> Rgba<u8> {
        match inner_color {
            InnerColor::None => {
                if water_logged {
                    let water_color = color_mul(self.water_basic.0, self.biome_color.get_water(biome));
                    basic.blend(&water_color)
                }
                basic
            },
            InnerColor::Water => color_mul(basic, self.biome_color.get_water(biome)),
            InnerColor::Grass => color_mul(basic, self.biome_color.get_grass(biome, height)),
            InnerColor::Foliage => color_mul(basic, self.biome_color.get_foliage(biome, height))
        }
    }

}

fn color_mul(mut a: Rgba<u8>, b: Rgb<u8>) -> Rgba<u8> {
    a[0] = (a[0] as u16 * b[0] as u16 / 255) as u8;
    a[1] = (a[1] as u16 * b[1] as u16 / 255) as u8;
    a[2] = (a[2] as u16 * b[2] as u16 / 255) as u8;
    a
}