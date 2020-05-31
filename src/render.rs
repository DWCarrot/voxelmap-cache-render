use std::io::Read;
use std::io::Seek;
use std::str::Split;
use std::convert::TryFrom;

use zip::ZipArchive;
use image::Pixel;
use image::Rgba;
use image::RgbaImage;

use super::color::biome::InnerColor;
use super::color::biome::Biome;
use super::color::ColorManager;
use super::color::BakedColorManager;

pub type GEResult<T> = Result<T, Box<dyn std::error::Error>>;

const TILESIZE: (u32, u32) = (256, 256);

pub struct LayerView<'a> {
    raw: &'a[u8],
}

impl<'a> LayerView<'a> {

    pub fn height(&self) -> u8 {
        self.raw[0]
    }

    pub fn blockstate_id(&self) -> u16 {
        ((self.raw[1] as u16) << 8) | (self.raw[2] as u16) 
    }

    pub fn light(&self) -> u8 {
        self.raw[3]
    }

    pub fn skylight(&self) -> u8 {
        (self.raw[3] & 0xF0) >> 4
    }

    pub fn blocklight(&self) -> u8 {
        (self.raw[3] & 0x0F) >> 0
    }
}

pub struct ElementView<'a> {
    raw: &'a[u8],
}

impl<'a> ElementView<'a> {

    pub fn surface(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[0..4] }
    }

    pub fn seafloor(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[4..8] }
    }

    pub fn transparent(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[8..12] }
    }

    pub fn foliage(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[12..16] }
    }

    // pub fn _(&self) -> u8 {
    //     self.raw[16]
    // }

    pub fn biome(&self) -> u8 {
        self.raw[17]
    }
}

pub struct TileView<'a> {
    raw: &'a[u8],
}

impl<'a> TileView<'a> {

    pub fn element(&self, x: u32, z: u32) -> ElementView<'a> {
        let index = ((x + z * TILESIZE.0) * 18) as usize;
        ElementView { raw: &self.raw[index .. index + 18] }
    }
}

pub struct BlockProps {

    pub air: bool,

    pub water: bool,

    pub waterlogged: bool,

    pub biome_color: InnerColor,
}

impl BlockProps {

    pub fn new() -> Self {
        BlockProps {
            air: true,
            water: false,
            waterlogged: false,
            biome_color: InnerColor::None,
        }
    }

    pub fn new_from<'a, I: Iterator<Item = &'a str>>(name: &'a str, state: I) -> Self {
        let mut waterlogged = false;
        for s in state {
            let mut it = s.split('=');
            if it.next() == Some("waterlogged") {
                if it.next() == Some("true") {
                    waterlogged = true;
                }
            }
        }
        BlockProps {
            air: name == "minecraft:air",
            water: name == "minecraft:water",
            waterlogged,
            biome_color: InnerColor::from(name)
        }
    }
}


pub struct Tile {

    id: (i32, i32),

    data: Vec<u8>,

    key: Vec<(Rgba<u8>, BlockProps)>,

}

impl Tile {

    pub fn load<R: Read + Seek>(reader: R, id: (i32, i32), mgr: &BakedColorManager) -> GEResult<Self> {
        let mut zip = ZipArchive::new(reader).map_err(Box::new)?;
        let mut data = Vec::new();
        let n = zip.by_name("data").map_err(Box::new)?.read_to_end(&mut data).map_err(Box::new)?;
        if n != TILESIZE.0 as usize * TILESIZE.1 as usize * 18 {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "data")))
        }
        let mut key = Vec::new();
        let mut key_string = String::new();
        let n = zip.by_name("key").map_err(Box::new)?.read_to_string(&mut key_string).map_err(Box::new)?;
        for line in key_string.lines() {
            match KeyLine::try_from(line) {
                Ok(k) => {
                    let props = BlockProps::new_from(k.name, SplitIter::from(k.state));
                    let model = mgr.get_basic_color(k.name, SplitIter::from(k.state), props.waterlogged);
                    
                    key.push((model, props));
                },
                Err(e) => {
                    eprintln!("parse error: `{}` @{}", line, e); //TODO: log
                    key.push((Rgba::from([0,0,0,0]), BlockProps::new()))
                }
            }
        }
        Ok(Tile {
            id,
            data,
            key
        })
    }

    pub fn view<'a>(&'a self) -> TileView<'a> {
        TileView { raw: self.data.as_slice() }
    }

    pub fn get_color(&self, id: u16) -> &(Rgba<u8>, BlockProps) {
        &self.key[(id - 1) as usize]
    }
}



#[derive(Debug)]
pub struct KeyLine<'a> {
    pub id: usize,
    pub name: &'a str,
    pub state: Option<&'a str>,
}


impl<'a> TryFrom<&'a str> for KeyLine<'a> {
    type Error = usize;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut state = None;

        let mut p = value;
        let mut pos = 0;

        let i = p.find(' ').ok_or(pos)?;
        let id = p[0..i].parse().map_err(|e| pos)?;
        p = &p[i+1..];
        pos += i + 1;

        let i = p.find('{').ok_or(pos)?;
        let s = &p[0..i];
        if s != "Block" {
            return Err(pos);
        }
        p = &p[i+1..];
        pos += i + 1;

        let i = p.find('}').ok_or(pos)?;
        let name = &p[0..i];
        p = &p[i+1..];
        pos += i + 1;

        if p.starts_with('[') {
            p = &p[1..];
            pos += 1;
            let i = p.find(']').ok_or(pos)?;
            state = Some(&p[0..i]);
        }

        Ok(KeyLine {
            id,
            name,
            state
        })
    }
}

pub struct SplitIter<'a>(Option<Split<'a, char>>);

impl<'a> From<Option<&'a str>> for SplitIter<'a> {

    fn from(value: Option<&'a str>) -> Self {
        SplitIter(value.map(|s| s.split(',')))
    }
}

impl<'a> Iterator for SplitIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(it) = &mut self.0 {
            it.next()
        } else {
            None
        }
    }
}




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