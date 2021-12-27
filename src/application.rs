use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::fs;
use std::fs::File;
use std::sync::Arc;
use std::thread;


use super::color::de;
use super::color::BakedColorManager;
use super::render;
use super::render::RenderOptions;
use super::render::tile::Tile;

pub struct AppOptions {
    render_options: RenderOptions,  
    input_folder: PathBuf,
    output_folder: PathBuf,
    thread_num: usize,
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

impl AppOptions {

    pub fn render_option_mut(&mut self) -> &mut RenderOptions {
        &mut self.render_options
    }

    pub fn set_thread_num(&mut self, thread_num: usize) {
        self.thread_num = thread_num;
    }

    pub fn set_input_folder(&mut self, path: &str) {
        self.input_folder = PathBuf::from(path);
    }

    pub fn set_output_folder(&mut self, path: &str) {
        self.output_folder = PathBuf::from(path);
    }

    pub fn ensure_output_folder(&self) -> io::Result<()> {
        if !self.output_folder.is_dir() {
            std::fs::create_dir_all(self.output_folder.as_path())
        } else {
            Ok(())
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

    pub fn alloc_tasks(this: Arc<Self> ,mut tasks: Vec<RenderTask>) {
        let thread_num = this.options.thread_num;
        let divide = std::cmp::max((tasks.len() + thread_num - 1) / thread_num, 1);
        let mut i = 0;
        let mut ths = Vec::new();
        let mut c = 0;
        while i < tasks.len() {
            let j = std::cmp::min(i + divide, tasks.len());
            let slice = Vec::from(&tasks[i..j]);
            let that = this.clone();
            let th = thread::Builder::new()
                .name(format!("work-{}", c))
                .spawn(move|| {
                for task in &slice {
                    if let Err(e) = that.render_one(task.src.as_path(), task.tgt.as_path(), &task.tile_id) {
                        log::warn!("[{}] tile{:?} error: {}", thread::current().name().unwrap_or_default(), task.tile_id, e);
                    } else {
                        log::info!("[{}] tile{:?} finished", thread::current().name().unwrap_or_default(), task.tile_id);
                    }
                }
            }).unwrap();
            ths.push(th);
            i = j;
            c += 1;
        }
        tasks.clear();
        for th in ths {
            th.join().unwrap();
        }
    }
}

pub fn curdir() -> PathBuf {
    use std::env::current_exe;

    if let Ok(dir) = current_exe() {
        if let Some(path) = dir.parent() {
            PathBuf::from(path)
        } else {
            PathBuf::default()
        }
    } else {
        PathBuf::default()
    }
}

pub fn build_colormanager() -> BakedColorManager {
    
    use std::io::BufReader;
    
    let mut dir = curdir();
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