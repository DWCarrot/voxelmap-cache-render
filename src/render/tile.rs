use std::io::Read;
use std::io::Seek;
use std::convert::TryFrom;

use log;
use zip::ZipArchive;
use image::Rgba;

use crate::color::ColorManager;
use crate::color::BakedColorManager;
use super::data::TILESIZE;
use super::data::View;
use super::data::V1TileView;
use super::data::V2TileView;
use super::data::ElementNode;
use super::data::LayerNode;
use super::key::BlockProps;
use super::key::KeyLine;
use super::key::SplitIter;
use super::control::Control;


pub type GEResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Tile {

    id: (i32, i32),

    data: Vec<u8>,

    key: Vec<(Rgba<u8>, BlockProps)>,

    control: Control,
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
                    log::warn!("parse error: `{}` @{}", line, e); //TODO: log
                    key.push((Rgba::from([0,0,0,0]), BlockProps::new()))
                }
            }
        }

        let mut control = Control::default();
        if let Ok(mut ifile) = zip.by_name("control") {
            let mut s = String::new();
            if let Ok(n) = ifile.read_to_string(&mut s) {
                for line in s.lines() {
                    if let Err(e) = control.modify_by(line) {
                        log::warn!("parse error: `{}` @{}", line, e);
                        break;
                    }
                }
            }
        }

        Ok(Tile {
            id,
            data,
            key,
            control
        })
    }

    pub fn view<'a>(&'a self) -> Box<dyn View<'a, EN=ElementNode<'a>, LN=LayerNode<'a>> + 'a> {
        match self.control.version {
            1 => Box::new(V1TileView::bind(self.data.as_slice())),
            2 => Box::new(V2TileView::bind(self.data.as_slice())),
            _ => {
                log::warn!("unexpected control{{version:{}}} , use `1`", self.control.version);
                Box::new(V1TileView::bind(self.data.as_slice()))
            }
        }   
    }

    pub fn get_color(&self, id: u16) -> &(Rgba<u8>, BlockProps) {
        &self.key[(id - 1) as usize]
    }
}