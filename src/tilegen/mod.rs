pub mod pathgen;
pub mod tile;

use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::Builder;

use image::imageops::FilterType;

use tile::LoadableImage;
use tile::TileId;
use tile::TileQTreeIterator;
use pathgen::PathGenerator;

pub fn merge_branch(root: TileId, cache: &mut HashMap<TileId, LoadableImage>, path_gen: &dyn PathGenerator, filter: FilterType) {
    for tile_id in TileQTreeIterator::new(root, 0) {
        if tile_id.is_origin() {
            if let Some(img) = cache.get_mut(&tile_id) {
                img.ensure();
                let p = path_gen.generate(tile_id.x, tile_id.z, tile_id.scale);
                match img.save(&p) {
                    Err(e) => log::warn!("tile {} @{} fail: {}", tile_id, p.display(), e),
                    Ok(b) => if b {
                        log::info!("tile {} generated", tile_id);
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
            match img.save(&p) {
                Err(e) => log::warn!("tile {} @{} fail: {}", tile_id, p.display(), e),
                Ok(b) => if b {
                    log::info!("tile {} generated", tile_id);
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
}

impl TileGeneratorOptions {

    pub fn new(input_folder: PathBuf, output_folder: PathBuf) -> Self {
        TileGeneratorOptions {
            filter: FilterType::Nearest,
            multi_thread_mode: false,
            input_folder,
            output_folder,
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

    pub fn build_path_generator(&self, mode: &str) -> Option<Arc<dyn PathGenerator + Send + Sync>> {
        use pathgen::Layer;

        let mut sp = mode.splitn(2, ':');
        match sp.next()? {
            "layer" => {
                let mut sp = sp.next()?.splitn(3, ',');
                let start: i32 = sp.next()?.parse().ok()?;
                let step: i32 = sp.next()?.parse().ok()?;
                let stop: i32 = sp.next()?.parse().ok()?;
                Some(Arc::new(Layer::new(start, step, stop, self.output_folder.to_path_buf())))
            }
            _ => None,
        }
    }
}


pub struct TileGenerator {
    options: TileGeneratorOptions,
    path_gen: Arc<dyn PathGenerator + Send + Sync>,
}

impl TileGenerator {

    pub fn new(options: TileGeneratorOptions, path_gen: Arc<dyn PathGenerator + Send + Sync>) -> Self {
        TileGenerator {
            options,
            path_gen
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

    pub fn generate_tile(&self, mut cache: HashMap<TileId, LoadableImage>) {
        let mut parts = vec![
            (-1, -1, HashMap::new()), 
            (0, -1, HashMap::new()), 
            (-1, 0, HashMap::new()), 
            (0, 0, HashMap::new())
        ];
        for (tile, image) in cache.into_iter() {
            parts[tile.side()].2.insert(tile, image);
        }
        let mut ths = Vec::new();
        for (x, z, mut cache_part) in parts.into_iter() {
            let path_gen = self.path_gen.clone();
            let root = TileId::new(path_gen.get_max_scale(), x, z);
            let filter = self.options.filter.clone();
            let tasks = move || {
                let path_gen = path_gen.as_ref();
                merge_branch(root, &mut cache_part, path_gen, filter)
            };
            if self.options.multi_thread_mode {
                let th = Builder::new().name(format!("work-({},{})", x, z)).spawn(tasks).unwrap();
                ths.push(th);
            } else {
                tasks();
            }
        }
        for th in ths {
            th.join().unwrap();
        }
    }
}