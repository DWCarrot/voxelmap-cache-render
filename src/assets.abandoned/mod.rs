pub mod data_type;
pub mod data_raw;
pub mod blockstate;
pub mod model;
pub mod loader;

use std::rc::Rc;
use std::collections::hash_map::HashMap;

use image::RgbaImage;
use image::RgbImage;

use model::TransformedModel;
use blockstate::BlockState;


enum LazyImage {

    Empty,

    Name(String),

    ImageRGB(Rc<RgbImage>),

    ImageRGBA(Rc<RgbaImage>),

}

impl Default for LazyImage {

    fn default() -> Self {
        Self::Empty
    }
}


pub struct ModelProvider {

    cache: HashMap<String, BlockState<String, TransformedModel<LazyImage>>>,

}


impl ModelProvider {
    
}

