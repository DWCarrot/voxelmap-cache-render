use std::rc::Rc;
use std::collections::btree_map::BTreeMap;
use std::io;
use std::fmt::Debug;

use super::data_type::Face;
use super::data_type::Rotate90;
use super::data_raw::FaceTextureRaw;
use super::data_raw::ElementRaw;
use super::data_raw::ModelRaw;
use super::data_raw::BlockStateRaw;
use super::data_raw::ApplyRaw;
use super::blockstate::BlockState;
use super::blockstate::Expression;


/**
 * 
 */

pub trait Provider {
    type Item;

    fn get(&mut self, name: &str) -> Option<Self::Item>;

}


#[derive(Debug)]
pub struct TransformedModel<Tex> {

    pub model: Rc<Model<Tex>>,

    pub x: Rotate90,

    pub y: Rotate90,

    pub uvlock: bool,

}

impl<Tex> TransformedModel<Tex> {

    pub fn from_mxy(model: Rc<Model<Tex>>, x: Rotate90, y: Rotate90, uvlock: bool) -> Self {
        TransformedModel {
            model,
            x,
            y,
            uvlock
        }
    }

    pub fn mapping(&self, face: Face) -> Face {
        FACE_ROTATE[self.x.index()][self.y.index()][face.index()].clone()
    }

    pub fn inv_mapping(&self, face: Face) -> Face {
        FACE_ROTATE_INV[self.x.index()][self.y.index()][face.index()].clone()
    }

    pub fn rotation(&self, original_face: Face) -> Rotate90 {
        let rotate = if !self.uvlock {
            Rotate90::R0
        } else {
            match original_face {
                Face::Up => self.y.clone(),
                Face::Down => -self.y.clone(),
                Face::East => -self.x.clone(),
                Face::West => self.x.clone(),
                Face::South => Rotate90::R0,
                Face::North => Rotate90::R0,
            }
        };
        -rotate
    }

}


#[derive(Debug)]
pub struct FaceTexture<Tex> {

    pub uv: [f32; 4],

    pub cullface: Option<Face>,

    pub rotation: Rotate90,

    pub texture: Tex,

    pub tintindex: Option<usize>,

}

impl<Tex: Default> FaceTexture<Tex> {

    pub fn from_raw<'a>(raw: &FaceTextureRaw, tex_gen: &'a mut dyn Provider<Item = Tex>) -> Self {
        FaceTexture {
            uv: {
                match &raw.uv {
                    Some(a) => a.as_ref().clone(),
                    None => [0.0, 0.0, 16.0, 16.0],
                }
            },
            cullface: raw.cullface.clone(),
            texture: tex_gen.get(raw.texture.as_str()).unwrap_or_default(),
            rotation: raw.rotation.clone().unwrap_or_else(|| Rotate90::R0),
            tintindex: raw.tintindex,
        }
    } 

}


#[derive(Debug)]
pub struct Model<Tex> {

    pub ambientocclusion: bool,

    pub elements: Vec<Element<Tex>>,
}

impl<Tex: Default> Model<Tex> {

    pub fn from_raw<'a>(raw: &ModelRaw, tex_gen: &'a mut dyn Provider<Item = Tex>) -> Self {
        Model {
            ambientocclusion: raw.ambientocclusion,
            elements: {
                let mut elements = Vec::with_capacity(raw.elements.len());
                for element in &raw.elements {
                    elements.push(Element::from_raw(element, tex_gen))
                }
                elements
            }
        }
    }
}


#[derive(Debug)]
pub struct Element<Tex> {

    pub from: [f32; 3],

    pub to: [f32; 3],

    pub faces: [Option<FaceTexture<Tex>>; 6],

    pub shade: bool,
    
}

impl<Tex: Default> Element<Tex> {

    pub fn from_raw<'a>(raw: &ElementRaw, tex_gen: &'a mut dyn Provider<Item = Tex>) -> Self {
        Element {
            
            from: raw.from,
            to: raw.to,
            shade: raw.shade,
            faces: {
                let mut faces = [None, None, None, None, None, None];
                for (k, v) in &raw.faces {
                    faces[k.index()] = Some(FaceTexture::from_raw(v, tex_gen));
                }
                faces
            }
        }
    }

    pub fn get_area(&self, face: Face) -> f32 {
        match face {
            Face::Down  | Face::Up    => (self.to[2] - self.from[2]) * (self.to[0] - self.from[0]),
            Face::East  | Face::West  => (self.to[2] - self.from[2]) * (self.to[1] - self.from[1]),
            Face::North | Face::South => (self.to[1] - self.from[1]) * (self.to[0] - self.from[0]),
        }
    }
}



// notice that x-rotate is inv-clockwise(right hand) and y-rotate is clockwise
pub const FACE_ROTATE: [[[Face;6];4];4] = [
    [
        [Face::West,Face::Down,Face::North,Face::South,Face::Up,Face::East],
        [Face::South,Face::Down,Face::West,Face::East,Face::Up,Face::North],
        [Face::East,Face::Down,Face::South,Face::North,Face::Up,Face::West],
        [Face::North,Face::Down,Face::East,Face::West,Face::Up,Face::South]
    ],
    [
        [Face::West,Face::South,Face::Down,Face::Up,Face::North,Face::East],
        [Face::Up,Face::South,Face::West,Face::East,Face::North,Face::Down],
        [Face::East,Face::South,Face::Up,Face::Down,Face::North,Face::West],
        [Face::Down,Face::South,Face::East,Face::West,Face::North,Face::Up]
    ],
    [
        [Face::West,Face::Up,Face::South,Face::North,Face::Down,Face::East],
        [Face::North,Face::Up,Face::West,Face::East,Face::Down,Face::South],
        [Face::East,Face::Up,Face::North,Face::South,Face::Down,Face::West],
        [Face::South,Face::Up,Face::East,Face::West,Face::Down,Face::North]
    ],
    [
        [Face::West,Face::North,Face::Up,Face::Down,Face::South,Face::East],
        [Face::Down,Face::North,Face::West,Face::East,Face::South,Face::Up],
        [Face::East,Face::North,Face::Down,Face::Up,Face::South,Face::West],
        [Face::Up,Face::North,Face::East,Face::West,Face::South,Face::Down]
    ]
];

pub const FACE_ROTATE_INV: [[[Face;6];4];4] = [
    [
        [Face::West,Face::Down,Face::North,Face::South,Face::Up,Face::East],
        [Face::North,Face::Down,Face::East,Face::West,Face::Up,Face::South],
        [Face::East,Face::Down,Face::South,Face::North,Face::Up,Face::West],
        [Face::South,Face::Down,Face::West,Face::East,Face::Up,Face::North]
    ],
    [
        [Face::West,Face::North,Face::Up,Face::Down,Face::South,Face::East],
        [Face::North,Face::East,Face::Up,Face::Down,Face::West,Face::South],
        [Face::East,Face::South,Face::Up,Face::Down,Face::North,Face::West],
        [Face::South,Face::West,Face::Up,Face::Down,Face::East,Face::North]
    ],
    [
        [Face::West,Face::Up,Face::South,Face::North,Face::Down,Face::East],
        [Face::North,Face::Up,Face::West,Face::East,Face::Down,Face::South],
        [Face::East,Face::Up,Face::North,Face::South,Face::Down,Face::West],
        [Face::South,Face::Up,Face::East,Face::West,Face::Down,Face::North]
    ],
    [
        [Face::West,Face::South,Face::Down,Face::Up,Face::North,Face::East],
        [Face::North,Face::West,Face::Down,Face::Up,Face::East,Face::South],
        [Face::East,Face::North,Face::Down,Face::Up,Face::South,Face::West],
        [Face::South,Face::East,Face::Down,Face::Up,Face::West,Face::North]
    ]
];


#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct KVPair {

    pub key: String,

    pub val: String,

}

impl std::fmt::Debug for KVPair {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.val)
    }
}


/**
 * 
 */

struct IndexTexGen<'a, Tex> {
    index: &'a BTreeMap<String, String>,
    tex_gen: &'a mut dyn Provider<Item = Tex>,
}

impl<'a, Tex> Provider for IndexTexGen<'a, Tex> {
    type Item = Tex;

    fn get(&mut self, name: &str) -> Option<Self::Item> {
        let mut u = name;
        while u.starts_with('#') {
            u = &u[1..];
            u = match self.index.get(u) {
                Some(v) => v,
                None => return self.tex_gen.get(name),
            };
        }
        self.tex_gen.get(u)
    }
}


pub struct BlockModelBuilder<'a, Tex> {

    bs_pvd: &'a mut dyn Provider<Item = BlockStateRaw>,

    mdl_pvd: &'a mut dyn Provider<Item = ModelRaw>,

    tex_gen: &'a mut dyn Provider<Item = Tex>,

    mdl_cache: BTreeMap<String, Rc<Model<Tex>>>,
}

impl<'a, Tex: Default> BlockModelBuilder<'a, Tex> {

    pub fn new(
        bs_pvd: &'a mut dyn Provider<Item = BlockStateRaw>,
        mdl_pvd: &'a mut dyn Provider<Item = ModelRaw>,
        tex_gen: &'a mut dyn Provider<Item = Tex>,
    ) -> Self {
        BlockModelBuilder {
            bs_pvd,
            mdl_pvd,
            tex_gen,
            mdl_cache: BTreeMap::new()
        }
    }

    pub fn build(&mut self, name: &str) -> io::Result<BlockState<String, Rc<TransformedModel<Tex>>>> {
        use std::collections::btree_map::Entry;
        use crate::assets::data_raw::Merge;

        let mdl_pvd = &mut self.mdl_pvd;
        let tex_gen = &mut self.tex_gen;
        let mdl_cache = &mut self.mdl_cache;
        let mut transf_apply = |v: ApplyRaw| -> io::Result<Rc<TransformedModel<Tex>>> {
            let v = v.get_fast();
            
            let model = match mdl_cache.entry(v.model.clone()) {
                Entry::Occupied(oc) => oc.get().clone(),
                Entry::Vacant(vc) => {
                    let mut mdl_raw = mdl_pvd.get(vc.key()).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, vc.key().to_string()))?;
                    while let Some(s) = &mdl_raw.parent {
                        let parent = mdl_pvd.get(s).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, s.to_string()))?;
                        mdl_raw.merge(&parent);
                    }
                    let mut itex_gen = IndexTexGen { 
                        index: mdl_raw.textures.as_ref().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "texture"))?, 
                        tex_gen: *tex_gen
                    };
                    let rcmodel = Rc::new(Model::from_raw(&mdl_raw, &mut itex_gen));
                    if rcmodel.elements.len() == 0 {
                        println!("empty model: {}", v.model);
                    }
                    vc.insert(rcmodel).clone()
                }
            };
            Ok(Rc::new(TransformedModel::from_mxy(model, v.x.clone(), v.y.clone(), v.uvlock)))
        };

        if let Some(bs_raw) = self.bs_pvd.get(name) {
            match bs_raw {
                BlockStateRaw::Variants(v_raw) => {
                    let mut blockstate = BlockState::Variants(Expression::default());
                    for(keys, apply_raw) in v_raw.into_iter() {
                        blockstate.insert_group(keys.iter(), transf_apply(apply_raw)?);
                    }
                    Ok(blockstate.try_simplify_variant())
                },
                BlockStateRaw::MultiPart(m_raw) => {
                    let mut blockstate = BlockState::MultiPart(Expression::default(), Vec::default());
                    for(mut group, apply_raw) in m_raw.into_iter() {
                        let model = transf_apply(apply_raw)?;
                        blockstate.start_group();
                        loop {
                            let it = group.line_iter();
                            blockstate.insert_group(it, model.clone());
                            if !group.update_iter() {
                                break;
                            }
                        }
                    }
                    Ok(blockstate)
                }
            }
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, name.to_string()))
        }      
    }

    // pub fn build_water_model(&mut self) -> Rc<TransformedModel<Tex>> {
    //     let model = Rc::new(Model {
    //         ambientocclusion: true,
    //         elements: vec![
    //             Element {
    //                 from: [0.0, 0.0, 0.0],
    //                 to:  [16.0, 16.0, 16.0],
    //                 shade: false,
    //                 faces: [
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/water_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     None,
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/water_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/water_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/water_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/water_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                 ]
    //             }
    //         ]
    //     });
    //     let tmodel = Rc::new(TransformedModel {
    //         model,
    //         uvlock: false,
    //         x: Rotate90::R0,
    //         y: Rotate90::R0,
    //     });
    //     tmodel
    // }

    // pub fn build_lava_model(&mut self) -> Rc<TransformedModel<Tex>> {
    //     let model = Rc::new(Model {
    //         ambientocclusion: true,
    //         elements: vec![
    //             Element {
    //                 from: [0.0, 0.0, 0.0],
    //                 to:  [16.0, 16.0, 16.0],
    //                 shade: false,
    //                 faces: [
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/lava_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     None,
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/lava_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/lava_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/lava_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                     Some(FaceTexture {
    //                         uv: [0.0, 0.0, 16.0, 16.0],
    //                         cullface: None,
    //                         rotation: Rotate90::R0,
    //                         texture: self.tex_gen.get("block/lava_still"),
    //                         tintindex: Some(0)
    //                     }),
    //                 ]
    //             }
    //         ]
    //     });
    //     let tmodel = Rc::new(TransformedModel {
    //         model,
    //         uvlock: false,
    //         x: Rotate90::R0,
    //         y: Rotate90::R0,
    //     });
    //     tmodel
    // }
}