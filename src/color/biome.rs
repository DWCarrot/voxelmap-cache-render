use image::Rgb;
use image::RgbImage;
use image::DynamicImage;

#[derive(Debug, Clone)]
pub struct Biome(pub usize);


const SEA_LEVEL: i32 = 63;


pub enum BiomeColorTOps {
    None,
    Fixed(Rgb<u8>),
    Average(Rgb<u8>)
}

impl BiomeColorTOps {

    pub fn modify(&self, color: &Rgb<u8>) -> Rgb<u8> {
        match self {
            BiomeColorTOps::None => {
                color.clone()
            },
            BiomeColorTOps::Fixed(fixed) => {
                fixed.clone()
            },
            BiomeColorTOps::Average(base) => {
                Rgb::from([
                    ((color[0] as u16 + base[0] as u16) / 2) as u8,
                    ((color[1] as u16 + base[1] as u16) / 2) as u8,
                    ((color[2] as u16 + base[2] as u16) / 2) as u8,
                ])
            },
        }
    }

}


fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        return min
    }
    if value > max {
        return max
    }
    value
}

pub struct BiomeProps {
    temperature: f32,
    rainfall: f32,
}

impl BiomeProps {

    pub fn new(temperature:f32, rainfall: f32) -> Self {
        BiomeProps {
            temperature,
            rainfall,
        }
    }

    pub fn adjust(&self, height: i32) -> Self {
        let temperature = self.temperature - (height - SEA_LEVEL) as f32 / 600.0;
        let temperature = clamp(temperature, 0.0, 1.0);
        let rainfall = clamp(self.rainfall, 0.0, 1.0) * temperature;
        BiomeProps {
            temperature,
            rainfall
        }
    }
}

pub struct BiomeColor {

    biomes: Vec<(String, BiomeProps, Rgb<u8>, BiomeColorTOps, BiomeColorTOps)>,

    grass: RgbImage,

    foliage: RgbImage,

}

impl BiomeColor {

    #[inline]
    fn get<'a, T>(vec: &'a Vec<T>, biome: &Biome) -> &'a T {
        if biome.0 < vec.len() {
            &vec[biome.0]
        } else {
            &vec[0]
        }
    }

    pub fn from_raw(biomes: Vec<(String, BiomeProps, Rgb<u8>, BiomeColorTOps, BiomeColorTOps)>, grass: RgbImage, foliage: RgbImage) -> Self {
        BiomeColor {
            biomes,
            grass,
            foliage,
        }
    }

    pub fn get_water(&self, biome: &Biome) -> Rgb<u8> {
        BiomeColor::get(&self.biomes, biome).2.clone()
    }

    pub fn get_grass(&self, biome: &Biome, height: i32) -> Rgb<u8> {
        let t = BiomeColor::get(&self.biomes, biome);
        let BiomeProps { temperature, rainfall } = t.1.adjust(height);
        let w = (self.grass.width() - 1) as f32;
        let h = (self.grass.height() - 1) as f32;
        let x = ((1.0 - temperature) * w).round() as u32;
        let y = ((1.0 - rainfall) * h).round() as u32;
        let c = self.grass.get_pixel(x, y);
        t.3.modify(c)
    }

    pub fn get_foliage(&self, biome: &Biome, height: i32) -> Rgb<u8> {
        let t = BiomeColor::get(&self.biomes, biome);
        let BiomeProps { temperature, rainfall } = t.1.adjust(height);
        let w = (self.foliage.width() - 1) as f32;
        let h = (self.foliage.height() - 1) as f32;
        let x = ((1.0 - temperature) * w).round() as u32;
        let y = ((1.0 - rainfall) * h).round() as u32;
        let c = self.foliage.get_pixel(x, y);
        t.4.modify(c)
    }
}


pub enum InnerColor {
    None,
    Water,
    Grass,
    Foliage,
}

impl From<&str> for InnerColor {

    fn from(value: &str) -> Self {
        match value {
            "minecraft:water" => Self::Water,
            "minecraft:grass_block" => Self::Grass,
            "minecraft:lily_pad" => Self::Grass,
            _ => {
                if value.ends_with("leaves") {
                    return Self::Foliage;
                }
                Self::None
            }
        }
    }
}
