use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use super::color::de;
use super::color::BakedColorManager;
use super::render;
use super::render::RenderOptions;
use super::render::Tile;

pub struct AppOptions {
    pub render_options: RenderOptions,  
    pub input_folder: PathBuf,
    pub output_folder: PathBuf,
    pub thread_num: usize,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            render_options: Default::default(),
            input_folder: Default::default(),
            output_folder: Default::default(),
            thread_num: 1
        }
    }
}


pub struct Application {
    options: AppOptions,
    color_mgr: BakedColorManager,
}

#[derive(Debug, Clone)]
pub struct RenderTask {
    pub src: PathBuf,
    pub tgt: PathBuf,
    pub tile_id: (i32, i32)
}

impl Application {

    pub fn new(options: AppOptions) -> Self {
        Application {
            color_mgr: build_colormanager(),
            options
        }
    }

    pub fn list_files(&self) -> Vec<RenderTask> {
        const EXT: &'static str = ".zip";
        if let Ok(read_dir) = self.options.input_folder.read_dir() {
            let odir = &self.options.output_folder;
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
                let tgt = odir.join(format!("{},{}.png", x, z));
                Some(RenderTask{src, tgt, tile_id: (x, z)})
            }).collect()
        } else {
            Vec::new()
        }
    }

    pub fn render_one(&self, src: &Path, tgt: &Path, tile_id: &(i32, i32)) -> Result<(), Box<dyn std::error::Error>> {
        use image::ImageFormat::Png;

        let ifile = File::open(src).map_err(Box::new)?; 
        let tile = Tile::load(ifile, tile_id.clone(), &self.color_mgr)?;
        let pic = render::render(tile, &self.color_mgr, &self.options.render_options);
        pic.save_with_format(tgt, Png).map_err(error_trans)
    }

    pub fn alloc_tasks(this: Arc<Self> ,tasks: Vec<RenderTask>) {
        let thread_num = this.options.thread_num;
        let divide = std::cmp::max((tasks.len() + thread_num - 1) / thread_num, 1);
        let mut i = 0;
        let mut ths = Vec::new();
        let time = Instant::now();
        while i < tasks.len() {
            let j = std::cmp::min(i + divide, tasks.len());
            let slice = Vec::from(&tasks[i..j]);
            let that = this.clone();
            let th = thread::spawn(move|| {
                println!("start [{}] @ {:?}", slice.len(), thread::current());
                for task in &slice {
                    if let Err(e) = that.render_one(task.src.as_path(), task.tgt.as_path(), &task.tile_id) {
                        eprintln!("task {:?} @{} error: {}", task.tile_id, task.src.display(), e);
                    }
                }
            });
            ths.push(th);
            i = j;
        }
        for th in ths {
            th.join().unwrap();
        }
        let time = Instant::now() - time;
        println!("> used: {}ms", time.as_millis());
    }
}



fn build_colormanager() -> BakedColorManager {
    use std::env::current_exe;
    use std::io::BufReader;
    
    let dir = current_exe().unwrap();
    let mut dir = PathBuf::from(dir.parent().unwrap());
    dir.push("resource");
    let biome_color = {
        let r4 = File::open(dir.join("biome.json")).unwrap();
        let r5 = File::open(dir.join("grass.png")).unwrap();
        let r6 = File::open(dir.join("foliage.png")).unwrap();
        de::build_biomecolor(r4, BufReader::new(r5), BufReader::new(r6))
    };
    {
        let r1 = File::open(dir.join("index.json")).unwrap();
        let r2 = File::open(dir.join("colormap.png")).unwrap();
        let r3 = File::open(dir.join("weightmap.png")).unwrap();
        de::build_backedcolormanager(r1, BufReader::new(r2), BufReader::new(r3), biome_color)
    }
}

fn error_trans(e: image::ImageError) -> Box<dyn std::error::Error> {
    use image::ImageError;

    match e {
        ImageError::IoError(err) => Box::new(err),
        ImageError::Decoding(err) => Box::new(err),
        ImageError::Encoding(err) => Box::new(err),
        ImageError::Parameter(err) => Box::new(err),
        ImageError::Limits(err) => Box::new(err),
        ImageError::Unsupported(err) => Box::new(err),
    }
}