use std::rc::Rc;
use std::cell::RefCell;
use std::io::Read;
use std::io::Seek;
use zip::read::ZipArchive;
use zip::read::ZipFile;
use zip::result::ZipResult;
use serde_json;
use image;
use image::RgbaImage;

use super::data_raw::ModelRaw;
use super::data_raw::BlockStateRaw;
use super::model::Provider;

/**
 * 
 */

pub struct ModelRawProvider<R: Read + Seek> {
    zip: Rc<RefCell<ZipArchive<R>>>,
    pub count: usize
}

impl<R: Read + Seek> From<Rc<RefCell<ZipArchive<R>>>> for ModelRawProvider<R> {

    fn from(zip: Rc<RefCell<ZipArchive<R>>>) -> Self {
        ModelRawProvider {
            zip,
            count: 0
        }
    }
}

impl<R: Read + Seek> Provider for ModelRawProvider<R> {
    type Item = ModelRaw;

    fn get(&mut self, name: &str) -> Option<Self::Item> {
        let full = format!("assets/minecraft/models/{}.json", name);
        match self.zip.borrow_mut().by_name(&full) {
            Ok(v) => match serde_json::from_reader(v) {
                Ok(v) => {
                    self.count += 1;
                    Some(v)
                },
                Err(e) => {
                    //TODO: log
                    return None 
                }
            },
            Err(e) => {
                //TODO: log
                return None 
            }
        }
    }
}


/**
 * 
 */

pub struct BlockStateRawProvider<R: Read + Seek> {
    zip: Rc<RefCell<ZipArchive<R>>>,
    pub count: usize
}

impl<R: Read + Seek> From<Rc<RefCell<ZipArchive<R>>>> for BlockStateRawProvider<R> {

    fn from(zip: Rc<RefCell<ZipArchive<R>>>) -> Self {
        BlockStateRawProvider {
            count: 0,
            zip
        }
    }
}

impl<R: Read + Seek> Provider for BlockStateRawProvider<R> {
    type Item = BlockStateRaw;

    fn get(&mut self, name: &str) -> Option<Self::Item> {
        let full = format!("assets/minecraft/blockstates/{}.json", name);
        match self.zip.borrow_mut().by_name(&full) {
            Ok(v) => match serde_json::from_reader(v) {
                Ok(v) => {
                    self.count += 1;
                    Some(v)
                },
                Err(e) => {
                    //TODO: log
                    None 
                }
            },
            Err(e) => {
                //TODO: log
                None 
            }
        } 
    }
}


/**
 * 
 */

pub struct TextureImageProvider<R: Read + Seek> {
    zip: Rc<RefCell<ZipArchive<R>>>,
    pub count: usize
}

impl<R: Read + Seek> From<Rc<RefCell<ZipArchive<R>>>> for TextureImageProvider<R> {

    fn from(zip: Rc<RefCell<ZipArchive<R>>>) -> Self {
        TextureImageProvider {
            count: 0,
            zip
        }
    }
}

// impl<'a, R: Read + Seek> Provider for TextureImageProvider<R> {
//     type Item = RgbaImage;

//     fn get(&mut self, name: &str) -> Option<Self::Item> {
//         let full = format!("assets/minecraft/textures/{}.mcmeta", name);
//         let animated = self.zip.borrow_mut().by_name(&full).is_ok();
//         let full = format!("assets/minecraft/textures/{}.png", name);
//         let img = match self.zip.borrow_mut().by_name(&full) {
//             Ok(v) => match image::png::PNGDecoder::new(v) {
//                 Ok(v) => match image::DynamicImage::from_decoder(v) {
//                     Ok(d) => match d {
//                         image::DynamicImage::ImageRgba8(v) => {
//                             self.count += 1;
//                             v
//                         },
//                         image::DynamicImage::ImageRgb8(v) => {
//                             self.count += 1;
//                             v.convert()
//                         },
//                         _ => {
//                             //TODO: log
//                             return None;
//                         }
//                     },
//                     Err(e) => {
//                         //TODO: log
//                         return None;
//                     }
//                 },
//                 Err(e) => {
//                     //TODO: log
//                     return None;
//                 }
//             },
//             Err(e) => {
//                 //TODO: log
//                 return None;
//             }
//         };
//         if animated {
//             let width = img.width();
//             let mut buf = img.into_raw();
//             buf.truncate(width as usize * width as usize * 4);
//             println!("animated: {} ({})", name, width);
//             Some(RgbaImage::from_vec(width, width, buf).unwrap())
//         } else {
//             Some(img)
//         }
//     }
// }
