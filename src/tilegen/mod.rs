pub mod pathgen;
pub mod tile;

use std::io;
use std::str::FromStr;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::thread::Builder;


use image::imageops::FilterType;

use tile::LoadableImage;
use tile::TileId;
use tile::TileQTreeIterator;
use pathgen::PathGenerator;

pub fn merge_branch(root: TileId, cache: &mut HashMap<TileId, LoadableImage>, path_gen: &dyn PathGenerator, filter: FilterType, check: bool) {
    for tile_id in TileQTreeIterator::new(root, 0) {
        if tile_id.is_origin() {
            if let Some(img) = cache.get_mut(&tile_id) {
                img.ensure();
                let p = path_gen.generate(tile_id.x, tile_id.z, tile_id.scale);
                match img.save(&p, check) {
                    Err(e) => {
                        log::warn!("[{}] tile {} @{} fail: {}", thread::current().name().unwrap_or_default(), tile_id, p.display(), e);
                    },
                    Ok(b) => {
                        if b {
                           log::info!("[{}] tile {} generated", thread::current().name().unwrap_or_default(), tile_id);
                        }
                    }
                }
            }
        } else {
            let bl = cache.remove(&tile_id.bottomleft()).unwrap_or_default();
            let br = cache.remove(&tile_id.bottomright()).unwrap_or_default();
            let tl = cache.remove(&tile_id.topleft()).unwrap_or_default();
            let tr = cache.remove(&tile_id.topright()).unwrap_or_default();
            let img = LoadableImage::merge(&tl, &tr, &bl, &br, filter);
            let p = path_gen.generate(tile_id.x, tile_id.z, tile_id.scale);
            match img.save(&p, check) {
                Err(e) => {
                    log::warn!("[{}] tile {} @{} fail: {}", thread::current().name().unwrap_or_default(), tile_id, p.display(), e);
                },
                Ok(b) => {
                    if b {
                       log::info!("[{}] tile {} generated", thread::current().name().unwrap_or_default(), tile_id);
                    }
                }
            }
            cache.insert(tile_id, img);  
        }
    }
}


pub struct TileGeneratorOptions {
    filter: FilterType,
    multi_thread_mode: bool,
    input_folder: PathBuf,
    output_folder: PathBuf,
    path_mode: PathMode,
    check: bool,
}

impl TileGeneratorOptions {

    pub fn new(input_folder: PathBuf, output_folder: PathBuf, path_mode: PathMode) -> Self {
        TileGeneratorOptions {
            filter: FilterType::Nearest,
            multi_thread_mode: false,
            input_folder,
            output_folder,
            path_mode,
            check: false,
        }
    }

    pub fn set_filter(&mut self, filter: &str) {
        match filter {
            "nearest" => self.filter = FilterType::Nearest,
            "triangle" => self.filter = FilterType::Triangle,
            "gaussian" => self.filter = FilterType::Gaussian,
            "catmullrom" => self.filter = FilterType::CatmullRom,
            "lanczos3" => self.filter = FilterType::Lanczos3,
            _ => { }
        }
    }

    pub fn set_multi_thread_mode(&mut self, mode: bool) {
        self.multi_thread_mode = mode;
    }

    pub fn set_check_exist(&mut self, check: bool) {
        self.check = check;
    }

    // pub fn build_path_generator(&self, mode: &str) -> Option<Arc<dyn PathGenerator + Send + Sync>> {
    //     use pathgen::Layer;

    //     let mut sp = mode.splitn(2, ':');
    //     match sp.next()? {
    //         "layer" => {
    //             let mut sp = sp.next()?.splitn(3, ',');
    //             let start: i32 = sp.next()?.parse().ok()?;
    //             let step: i32 = sp.next()?.parse().ok()?;
    //             let stop: i32 = sp.next()?.parse().ok()?;
    //             Some(Arc::new(Layer::new(start, step, stop, self.output_folder.to_path_buf())))
    //         }
    //         _ => None,
    //     }
    // }
}


pub struct Bound {
    pub xmin: i32,
    pub xmax: i32,
    pub zmin: i32,
    pub zmax: i32
}

impl Bound {
    
    pub fn new() -> Self {
        Bound {
            xmin: 0,
            xmax: -1,
            zmin: 0,
            zmax: -1,
        }
    }

    pub fn extend(&mut self, tile: &TileId) {
        if tile.x < self.xmin {
            self.xmin = tile.x;
        }
        if tile.x > self.xmax {
            self.xmax = tile.x;
        }
        if tile.z < self.zmin {
            self.zmin = tile.z;
        }
        if tile.z > self.zmax {
            self.zmax = tile.z;
        }
    }
}


pub fn ceil_log2(x: i32) -> i32 {
    32 - (x - 1).leading_zeros() as i32
}

#[derive(Debug)]
pub enum PathMode {
    Layer {
        reverse: bool,
        min_zoom: Option<i32>,
        max_zoom: Option<i32>,
    },
    Tree {
        
    }
}

impl FromStr for PathMode {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mode_sp = s.splitn(2, ':');
        match mode_sp.next().unwrap() {
            "layer+" => {
                if let Some(params) = mode_sp.next() {
                    let mut params_sp = params.splitn(2, ',');
                    let min_zoom = if let Some(z) = params_sp.next() {
                        Some(z.parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, "minZoom"))?)
                    } else {
                        None
                    };
                    let max_zoom = if let Some(z) = params_sp.next() {
                        Some(z.parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, "maxZoom"))?)
                    } else {
                        None
                    };
                    Ok(Self::Layer { 
                        reverse: false, 
                        min_zoom,
                        max_zoom
                    })
                } else {
                    Ok(Self::Layer { 
                        reverse: false, 
                        min_zoom: None, 
                        max_zoom: None
                    })
                }
            },
            "layer-" => {
                if let Some(params) = mode_sp.next() {
                    let mut params_sp = params.splitn(2, ',');
                    let mut max_zoom = if let Some(z) = params_sp.next() {
                        Some(z.parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, "maxZoom"))?)
                    } else {
                        None
                    };
                    let min_zoom = if let Some(z) = params_sp.next() {
                        Some(z.parse().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, "minZoom"))?)
                    } else {
                        let tmp = max_zoom;
                        max_zoom = None;
                        tmp
                    };
                    Ok(Self::Layer { 
                        reverse: true, 
                        min_zoom,
                        max_zoom
                    })
                } else {
                    Ok(Self::Layer { 
                        reverse: true, 
                        min_zoom: None, 
                        max_zoom: None
                    })
                }
            },
            "tree" => {
                unimplemented!();
            },
            _ => {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported"))
            }
        }
        
    }
    
}

impl PathMode {

    pub fn extract(&self, bound: &Bound, root: &Path) -> Arc<dyn PathGenerator + Send + Sync> {
        use std::cmp::max;
        use pathgen::Layer;

        match self {
            Self::Layer { reverse, min_zoom, max_zoom } => {
                let min_zoom = min_zoom.unwrap_or(0);
                let max_zoom = if let Some(z) = max_zoom {
                    *z
                } else {
                    min_zoom + ceil_log2(max(max(bound.xmin.abs(), bound.xmax.abs()), max(bound.zmin.abs(), bound.zmax.abs())))
                };
                if *reverse {
                    Arc::new(Layer::new(max_zoom, -1, min_zoom, PathBuf::from(root)))
                } else {
                    Arc::new(Layer::new(min_zoom, 1, max_zoom, PathBuf::from(root)))
                }
            },
            Self::Tree { } => {
                unimplemented!()
            }
        }
    }
}


pub struct TileGenerator {
    options: TileGeneratorOptions,
}

impl TileGenerator {

    pub fn new(options: TileGeneratorOptions) -> Self {
        TileGenerator {
            options,
        }
    }

    pub fn list_files(&self) -> HashMap<TileId, LoadableImage> {
        const EXT: &'static str = ".png";
        if let Ok(read_dir) = self.options.input_folder.read_dir() {
            read_dir.filter_map(|entry| {
                let src = entry.ok()?.path();
                let filename = src.file_name()?.to_str()?;
                if !filename.ends_with(EXT) {
                    return None;
                }
                let filename = &filename[0 .. filename.len() - EXT.len()];
                let mut sp = filename.splitn(2, ',');
                let x: i32 = sp.next()?.parse().ok()?;
                let z: i32 = sp.next()?.parse().ok()?;
                Some((TileId::new(0, x, z), LoadableImage::new(src)))
            }).collect()
        } else {
            HashMap::new()
        }
    }

    pub fn generate_tile(&self, cache: HashMap<TileId, LoadableImage>) {
        let mut parts = vec![
            (-1, -1, HashMap::new()), 
            (0, -1, HashMap::new()), 
            (-1, 0, HashMap::new()), 
            (0, 0, HashMap::new())
        ];
        let mut bound = Bound::new();
        for (tile, image) in cache.into_iter() {
            bound.extend(&tile);
            parts[tile.side()].2.insert(tile, image);
        }
        let path_gen = self.options.path_mode.extract(&bound, self.options.output_folder.as_path());
        let mut ths = Vec::new();
        for (x, z, mut cache_part) in parts.into_iter() {
            if cache_part.len() > 0 {
                let path_gen = path_gen.clone();
                let root = TileId::new(path_gen.get_max_scale(), x, z);
                let filter = self.options.filter.clone();
                let check = self.options.check;
                let tasks = move || {
                    let path_gen = path_gen.as_ref();
                    merge_branch(root, &mut cache_part, path_gen, filter, check)
                };
                if self.options.multi_thread_mode {
                    let th = Builder::new().name(format!("work-({},{})", x, z)).spawn(tasks).unwrap();
                    ths.push(th);
                } else {
                    tasks();
                }
            }
        }
        for th in ths {
            th.join().unwrap();
        }
    }
}


mod test {

    #[test]
    fn test_path_mode_parse() {
        use super::Bound;
        use super::PathMode;
        use std::str::FromStr;
        use std::path::PathBuf;

        let mut b = Bound::new();
        b.xmin = -32;
        b.xmax = 50;
        b.zmin = -43;
        b.zmax = 50;

        let p = PathBuf::new();

        for input in &[
            "layer+",
            "layer+:0",
            "layer+:2",
            "layer+:2,6",
            "layer-",
            "layer-:0",
            "layer-:2",
            "layer-:6,2",
        ] {
            match PathMode::from_str(input) {
                Ok(m) => {
                    println!("{:?}", m);
                    let layer = m.extract(&b, &p);
                    println!("{:?}", layer.get_max_scale());
                },
                Err(e) => {
                    println!("{}", e);
                }
            } 
        }
    }

}