use std::path::PathBuf;


pub trait PathGenerator {

    fn get_max_scale(&self) -> i32;

    fn set_max_scale(&mut self, s: i32);
    
    fn generate(&self, x: i32, z: i32, scale: i32 /* scale level increase from 0 */) -> PathBuf;

}


pub struct Layer {
    root: PathBuf,
    start: i32,
    step: i32,
    max_scale: i32
}

impl Layer {

    pub fn new(start: i32, step: i32, stop: i32, root: PathBuf) -> Self {
        Layer {
            root,
            start,
            step,
            max_scale: (stop - start) / step
        }
    }
}

impl PathGenerator for Layer {

    fn get_max_scale(&self) -> i32 {
        self.max_scale
    }

    fn set_max_scale(&mut self, s: i32) {
        self.max_scale = s;
    }

    fn generate(&self, x: i32, z: i32, scale: i32) -> PathBuf {
        let zoom = self.start + self.step * scale;
        let mut pathbuf = self.root.to_path_buf();
        pathbuf.push(format!("{}", zoom));
        pathbuf.push(format!("{},{}.png", x, z));
        pathbuf
    }

}


pub struct Tree {
    root: PathBuf,
    max_scale: i32,
    topleft: String, 
    topright: String, 
    bottomleft: String, 
    bottomright: String
}

