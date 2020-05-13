use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::hash::Hash;

use image::DynamicImage;
use image::RgbaImage;
use image::ImageFormat;
use image::ImageResult;
use image::ImageError;
use image::imageops;
use image::imageops::FilterType;


pub enum LoadableImage {
    Unloaded(PathBuf),
    Image(RgbaImage),
    Empty,
}

impl Default for LoadableImage {

    fn default() -> Self {
        Self::Empty
    }
}

impl LoadableImage {

    pub fn new(path: PathBuf) -> Self {
        Self::Unloaded(path)
    }

    #[inline]
    pub fn is_image(&self) -> bool {
        match self {
            Self::Image(_) => true,
            _ => false,
        }
    }

    pub fn ensure(&mut self) {
        use std::io::BufReader;

        if let Self::Unloaded(path) = self {
            let img = if let Ok(ifile) = File::open(path.as_path()) {
                if let Ok(image) = image::load(BufReader::new(ifile), ImageFormat::Png) {
                    if let DynamicImage::ImageRgba8(image) = image {
                        if image.width() != image.height() {
                            Self::Empty
                        } else {
                            Self::Image(image)
                        }
                    } else {
                        Self::Empty
                    }
                } else {
                    Self::Empty
                }
            } else {
                Self::Empty
            };
            *self = img;
        }
    }

    pub fn save(&self, path: &PathBuf) -> ImageResult<bool> {
        use image::buffer::ConvertBuffer;
        use image::RgbImage;

        if let Self::Image(image) = self {
            let mut full = true;
            for pixel in image.pixels() {
                if pixel[3] < 255 {
                    full = false;
                    break;
                }
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(ImageError::from)?;
            }
            if full {
                let image: RgbImage = image.convert();
                image.save_with_format(path.as_path(), ImageFormat::Png)?;
            } else {
                image.save_with_format(path.as_path(), ImageFormat::Png)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn merge(topleft: &Self, topright: &Self, bottomleft: &Self, bottomright: &Self, filter: FilterType) -> Self {
        let refs = [topleft, topright, bottomleft, bottomright]; //00 01 10 11
        let mut w = 0;
        let mut h = 0;
        for p in refs.iter() {
            if let Self::Image(img) = *p {
                w = img.width();
                h = img.height();
                break;
            }
        }
        if w * h == 0 {
            Self::Empty
        } else {
            let mut combine = RgbaImage::new(2 * w, 2 * h);
            for (i, p) in refs.iter().enumerate() {
                if let Self::Image(img) = *p {
                    let i = i as u32;
                    let ox = ((i & 0x1) >> 0) * w;
                    let oy = ((i & 0x2) >> 1) * h;
                    imageops::replace(&mut combine, img, ox, oy);
                }
            }
            Self::Image(imageops::resize(&combine, w, h, filter))
        }
    }

}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TileId {
    pub scale: i32,
    pub x: i32,
    pub z: i32,
}

impl std::fmt::Display for TileId {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})-[{}]", self.x, self.z, self.scale))
    }
}

impl TileId {

    pub fn new(scale: i32, x: i32, z: i32) -> Self {
        TileId {
            scale,
            x, 
            z,
        }
    }

    pub fn topleft(&self) -> Self {
        TileId {
            x: self.x * 2 + 0,
            z: self.z * 2 + 0,
            scale: self.scale - 1,
        }
    }

    pub fn topright(&self) -> Self {
        TileId {
            x: self.x * 2 + 1,
            z: self.z * 2 + 0,
            scale: self.scale - 1,
        }
    }
        
    pub fn bottomleft(&self) -> Self {
        TileId {
            x: self.x * 2 + 0,
            z: self.z * 2 + 1,
            scale: self.scale - 1,
        }
    }

    pub fn bottomright(&self) -> Self {
        TileId {
            x: self.x * 2 + 1,
            z: self.z * 2 + 1,
            scale: self.scale - 1,
        }
    }
    
    pub fn parent(&self) -> Self {
        TileId {
            x: self.x / 2,
            z: self.z / 2,
            scale: self.scale + 1,
        }
    }

    pub fn is_origin(&self) -> bool {
        self.scale == 0
    }

    pub fn side(&self) -> usize {
        let mut i = 0;
        if self.x >= 0 {
            i |= 0x1;
        }
        if self.z >= 0 {
            i |= 0x2;
        }
        i   // [topleft,topright,bottomleft,bottomright]
    }
}


#[derive(Clone)]
struct TileQTreeIndexNode {
    tile_id: TileId,
    parent_index: isize,
    child_id: usize,
}

impl TileQTreeIndexNode {
    pub fn new(tile_id: TileId, parent_index: isize) -> Self {
        TileQTreeIndexNode {
            tile_id,
            parent_index,
            child_id: 0
        }
    }
}


pub struct TileQTreeIterator {
    stack: Vec<TileQTreeIndexNode>,
    stop_layer: i32,
}

impl TileQTreeIterator {

    pub fn new(root: TileId, stop_layer: i32) -> Self {
        let mut stack = Vec::new();
        stack.push(TileQTreeIndexNode::new(root, -1));
        TileQTreeIterator {
            stack,
            stop_layer
        }
    }
}

impl Iterator for TileQTreeIterator {
    type Item = TileId;

    fn next(&mut self) -> Option<Self::Item> { 
        loop {
            let TileQTreeIndexNode{ tile_id, parent_index, child_id } = self.stack.last()?.clone();
            let current_index = self.stack.len() as isize - 1;
            if tile_id.scale <= self.stop_layer || child_id > 3 {
                if parent_index >= 0 {
                    let parent = &mut self.stack[parent_index as usize];
                    parent.child_id += 1;
                }
                self.stack.pop();
                return Some(tile_id);
            } else {
                self.stack.push(TileQTreeIndexNode::new(tile_id.bottomright(), current_index));
                self.stack.push(TileQTreeIndexNode::new(tile_id.bottomleft(), current_index));
                self.stack.push(TileQTreeIndexNode::new(tile_id.topright(), current_index));
                self.stack.push(TileQTreeIndexNode::new(tile_id.topleft(), current_index));
            }
        }
    }
}




mod test {

    #[test]
    fn test_tile_qtree() {
        use super::{TileId, TileQTreeIterator};
        
        let root = TileId::new(3, 0, 0);
        for tid in TileQTreeIterator::new(root, 1) {
            println!("{:?}", tid);
        }
    }

}