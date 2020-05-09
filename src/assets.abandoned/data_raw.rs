use std::collections::btree_map::BTreeMap;
use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::Deserializer;
use serde::de::Visitor;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::ser::Serializer;
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use super::data_type::PrimaryType;
use super::data_type::Rotate90;
use super::data_type::Face;
use super::data_type::Axis;


/**
 * 
 */
fn default_bool_true() -> bool { true }
fn default_f32_1() -> f32 { 1.0 }
fn default_3f32_0() -> [f32; 3] { [0.0f32, 0.0, 0.0] }
fn default_3f32_1() -> [f32; 3] { [1.0f32, 1.0, 1.0] }
fn default_3f32_8() -> [f32; 3] { [8.0f32, 8.0, 8.0] }
fn default_3f32_16() -> [f32;3] { [16.0, 16.0, 16.0] }

/**
 * 
 */

macro_rules! option_override {
    ($t:ident,$s:ident,$($field:ident),+) => {
        $(
            if $t.$field.is_none() && $s.$field.is_some() {
                $t.$field = $s.$field.clone();
            }
        )+
    };
}

//type TProvider<T> = dyn Provider<Item=T>;

pub trait Merge {

    fn merge(&mut self, other: &Self)  -> bool;
}


/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModelRaw {

    #[serde(default)]
    pub parent: Option<String>,

    #[serde(default = "default_bool_true")]
    pub ambientocclusion: bool, //true

    #[serde(default)]
    pub display: Option<DisplayRaw>,

    #[serde(default)]
    pub elements: Vec<ElementRaw>,

    #[serde(default)]
    pub textures: Option<BTreeMap<String, String>>
}

impl Merge for ModelRaw {

    fn merge(&mut self, other: &Self)  -> bool {
        self.parent = other.parent.clone();
        if let Some(s) = &mut self.display {
            if let Some(o) = &other.display {
                s.merge(o);
            }
        } else {
            if other.display.is_some() {
                self.display = other.display.clone();
            }
        }
        if self.elements.len() == 0 && other.elements.len() > 0 {
            self.elements = other.elements.clone();
        }
        if let Some(s) = &mut self.textures {
            if let Some(o) = &other.textures {
                for (k, v) in o {
                    s.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
        } else {
            if other.display.is_some() {
                self.display = other.display.clone();
            }
        }
        true
    }
}


/**
 * 
 */
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DisplayRaw {

    #[serde(default)]
    pub thirdperson_righthand: Option<Transform>, 
    
    #[serde(default)]
    pub thirdperson_lefthand: Option<Transform>, 
    
    #[serde(default)]
    pub firstperson_righthand: Option<Transform>, 
    
    #[serde(default)]
    pub firstperson_lefthand: Option<Transform>, 
    
    #[serde(default)]
    pub gui: Option<Transform>, 
    
    #[serde(default)]
    pub head: Option<Transform>, 
    
    #[serde(default)]
    pub ground: Option<Transform>, 
    
    #[serde(default)]
    pub fixed: Option<Transform>,
}

impl Merge for DisplayRaw {

    fn merge(&mut self, other: &Self)  -> bool {
        option_override!(self, other, thirdperson_righthand, thirdperson_lefthand, firstperson_lefthand, firstperson_righthand, gui, head, ground, fixed);
        true
    }
}


/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transform {

    #[serde(default = "default_3f32_0")]
    pub rotation: [f32; 3], // [0, 0, 0]

    #[serde(default = "default_3f32_0")]
    pub translation: [f32; 3], // [0, 0, 0]

    #[serde(default = "default_3f32_1")]
    pub scale: [f32; 3], // [1, 1, 1]
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            rotation: default_3f32_0(),
            translation: default_3f32_0(),
            scale: default_3f32_1(),
        }
    }
}



/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ElementRaw {

    #[serde(default = "default_3f32_0")]
    pub from: [f32; 3],

    #[serde(default = "default_3f32_16")]
    pub to: [f32; 3],

    #[serde(default)]
    pub rotation: Option<Box<Rotation>>,

    #[serde(default = "default_bool_true")]
    pub shade: bool, // true

    pub faces: BTreeMap<Face, FaceTextureRaw>,
}



impl Merge for ElementRaw {

    fn merge(&mut self, other: &Self)  -> bool {
        use std::collections::btree_map::Entry;

        option_override!(self, other, rotation);
        for (k, v) in &other.faces {
            match self.faces.entry(k.clone()) {
                Entry::Occupied(entry) => {
                    entry.into_mut().merge(v);
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }
        true
    }
}


/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rotation {

    #[serde(default = "default_3f32_8")]
    pub origin: [f32; 3],   // [8, 8, 8]

    pub axis: Axis, // Y

    #[serde(default)]
    pub angle: f32, // 0

    #[serde(default)]
    pub rescale: bool,
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation {
            origin: default_3f32_8(),
            axis: Axis::Y,
            angle: Default::default(),
            rescale: Default::default()
        }
    }
}



/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FaceTextureRaw {

    #[serde(default)]
    pub uv: Option<Box<[f32; 4]>>, // [0, 0, 16, 16]

    pub texture: String,

    #[serde(default)]
    pub cullface: Option<Face>, // Side

    #[serde(default)]
    pub rotation: Option<Rotate90>,  // clockwise

    #[serde(default)]
    pub tintindex: Option<usize>,
}

impl Merge for FaceTextureRaw {
    fn merge(&mut self, other: &Self)  -> bool {
        option_override!(self, other, uv, cullface, tintindex, rotation);
        true
    }
}


/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppliedModelRaw {

    #[serde(default)]
    pub x: Rotate90,    // inv-clockwise(right hand)
    
    #[serde(default)]
    pub y: Rotate90,    // clockwise

    pub model: String,

    #[serde(default)]
    pub uvlock: bool,

    #[serde(default = "default_f32_1")]
    pub weight: f32, 
}


/**
 * 
 */
#[derive(Clone, Debug)]
pub enum ApplyRaw {
    Item(AppliedModelRaw),
    Array(Vec<AppliedModelRaw>),
}

impl Serialize for ApplyRaw {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ApplyRaw::Item(m) => serializer.serialize_newtype_struct("AppliedModelRaw", m),
            ApplyRaw::Array(l) => {
                let mut seq = serializer.serialize_seq(Some(l.len()))?;
                for element in l.as_slice() {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ApplyRaw {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        struct InnerVisitor;

        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = ApplyRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("{model, uvlock?, x?, y?, weight?}, [{...}]")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;
                let mut model = None;
                let mut uvlock = None;
                let mut weight = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "x" => x = map.next_value()?,
                        "y" => y = map.next_value()?,
                        "model" => model = Some(map.next_value()?),
                        "uvlock" => uvlock = map.next_value()?,
                        "weight" => weight = map.next_value()?,
                        _ => { }
                    }
                }
                Ok(ApplyRaw::Item(AppliedModelRaw {
                    x: x.unwrap_or_default(),
                    y: y.unwrap_or_default(),
                    model: model.ok_or_else(||de::Error::missing_field("model"))?,
                    uvlock: uvlock.unwrap_or_default(),
                    weight: weight.unwrap_or_else(default_f32_1),
                }))
                
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut list: Vec<AppliedModelRaw> = Vec::with_capacity(seq.size_hint().unwrap_or(4));
                while let Some(v) = seq.next_element()? {
                    list.push(v);
                }
                Ok(ApplyRaw::Array(list))
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                ApplyRaw::deserialize(deserializer)
            }
        }
        //const FIELDS: &'static [&'static str] = &["x", "y", "uvlock", "model", "weight"];
        deserializer.deserialize_any(InnerVisitor)
    }
}

impl ApplyRaw {

    // pub fn get(&self) -> &AppliedModelRaw {
    //     match self {
    //         Self::Item(p) => p,
    //         Self::Array(list) => {
    //             let x = rand::random::<f32>();
    //             let mut p = 0.0;
    //             for t in list {
    //                 p += t.weight;
    //                 if x < p {
    //                     return &t;
    //                 }
    //             }
    //             &list[list.len()-1]
    //         },
    //     }
    // }

    pub fn get_fast(&self) -> &AppliedModelRaw {
        match self {
            Self::Item(p) => p,
            Self::Array(list) => &list[0],
        }
    }

}

/**
 * 
 */
#[derive(Clone, Debug)]
pub struct VariantsRaw(Vec<(Vec<String>, ApplyRaw)>);

impl IntoIterator for VariantsRaw {
    type Item = (Vec<String>, ApplyRaw);
    type IntoIter = std::vec::IntoIter<(Vec<String>, ApplyRaw)>;

    fn into_iter(self) -> std::vec::IntoIter<(Vec<String>, ApplyRaw)> {
        self.0.into_iter()
    }
}

impl<'de> Deserialize<'de> for VariantsRaw {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        struct InnerVisitor;

        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = VariantsRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("{ [condition, ,]: ApplyRaw }")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut list = Vec::new();
                while let Some(s) = map.next_key::<String>()? {
                    let key: &str = s.as_str();
                    let v = map.next_value()?;
                    let k = if key == "" {
                        Vec::new()
                    } else {
                        key.split(',').map(|s| s.to_string()).collect()
                    };
                    list.push((k, v));
                }
                Ok(VariantsRaw(list))
            }
        }
        
        deserializer.deserialize_map(InnerVisitor)
    }
}



/**
 * 
 */
#[derive(Clone, Debug)]
pub struct MultiPartRaw(Vec<(CaseNode<Vec<CaseNode<String>>>, ApplyRaw)>);

impl IntoIterator for MultiPartRaw {
    type Item = (CaseNode<Vec<CaseNode<String>>>, ApplyRaw);
    type IntoIter = std::vec::IntoIter<(CaseNode<Vec<CaseNode<String>>>, ApplyRaw)>;

    fn into_iter(self) -> std::vec::IntoIter<(CaseNode<Vec<CaseNode<String>>>, ApplyRaw)> {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug)]
pub enum CaseNode<T> {
    Item(T),
    Array(Vec<T>, usize)
}

impl<T> CaseNode<Vec<CaseNode<T>>> {

    pub fn update_iter(&mut self) -> bool {
        match self {
            CaseNode::Item(v) => {
                CaseNode::update_list(v.as_mut_slice())
            }
            CaseNode::Array(list, index) => {
                if !CaseNode::update_list(list[*index].as_mut_slice()) {
                    *index += 1;
                    if *index < list.len() {
                        true
                    } else {
                        *index = 0;
                        false
                    }
                } else {
                    true
                }
            }
        }
    }

    pub fn line_iter<'a>(&'a self) -> CaseItemRefIter<'a, T> {
        match self {
            CaseNode::Item(v) => {
                CaseItemRefIter{ inner: v.iter() }
            }
            CaseNode::Array(list, index) => {
                let v = &list[*index];
                CaseItemRefIter{ inner: v.iter() }
            }
        }
    }

    fn update_list(list: &mut [CaseNode<T>]) -> bool {
        let mut success = false;
        for node in list {
            match node {
                CaseNode::Item(_) => { },
                CaseNode::Array(list, index) => {
                    *index += 1;
                    if *index < list.len() {
                        success = true;
                        break;
                    } else {
                        *index = 0;
                    }
                }
            }
        }
        success
    }
}



pub struct CaseItemRefIter<'a, T> {
    inner: std::slice::Iter<'a, CaseNode<T>>,
}

impl<'a, T> Iterator for CaseItemRefIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.inner.next() {
            match node {
                CaseNode::Item(v) => Some(v),
                CaseNode::Array(list, index) => list.get(*index)
            }
        } else {
            None
        }
    }
}

impl<'de> Deserialize<'de> for MultiPartRaw {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;
        use std::collections::btree_map::BTreeMap;

        enum Case {
            OR(Vec<BTreeMap<String, PrimaryType>>),
            AND(BTreeMap<String, PrimaryType>),
        }

        impl<'de> Deserialize<'de> for Case {

            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct InnerVisitorCase;

                impl<'de> Visitor<'de> for InnerVisitorCase {
                    type Value = Case;
        
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("{[k:string]:string} or OR:[{...}]")
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                    where
                        V: MapAccess<'de>,
                    {
                        let mut case = BTreeMap::new();
                        while let Some(key) = map.next_key()? {
                            if key == "OR" {
                                if case.len() > 0 {
                                    return Err(de::Error::duplicate_field("OR"));
                                }
                                let multi = map.next_value()?;
                                return Ok(Case::OR(multi));
                            }
                            let value = map.next_value()?;
                            case.insert(key, value);
                        }
                        Ok(Case::AND(case))
                    }
                }

                deserializer.deserialize_map(InnerVisitorCase)
            }
            
        }

        #[derive(Deserialize)]
        struct CaseTuple {
            when: Option<Case>,
            apply: ApplyRaw
        }

        struct InnerVisitor;

        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = MultiPartRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("[{when: {}, apply: <ApplyRaw>} ]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                
                let insert_case = |pair: BTreeMap<String, PrimaryType> | -> Result<Vec<CaseNode<String>>, A::Error> {
                    let mut r = Vec::new();
                    for (pk, pv) in pair {
                        let pv: String = pv.into();
                        let node = if pv.contains('|') {
                            let mut ks = Vec::new();
                            for s in pv.split('|') {
                                let k = pk.clone() + "=" + s;
                                ks.push(k);
                            }
                            CaseNode::Array(ks, 0)
                        } else {
                            CaseNode::Item(pk + "=" + pv.as_str())
                        };
                        r.push(node);
                    }
                    Ok(r)
                };

                let mut list = Vec::new();
                while let Some(t) = seq.next_element::<CaseTuple>()? {
                    if let Some(case) = t.when {
                        match case {
                            Case::AND(pair) => {
                                list.push((CaseNode::Item(insert_case(pair)?), t.apply))
                            }
                            Case::OR(or_list) => {
                                let mut r = Vec::new();
                                for pair in or_list.into_iter() {
                                    r.push(insert_case(pair)?);
                                }
                                list.push((CaseNode::Array(r, 0), t.apply))
                            }
                        }
                    } else {
                        list.push((CaseNode::Item(Vec::new()), t.apply))
                    }
                }
                Ok(MultiPartRaw(list))
            }
        }
        
        deserializer.deserialize_seq(InnerVisitor)
    }
}



/**
 * 
 */
#[derive(Clone, Debug)]
pub enum BlockStateRaw {
    Variants(VariantsRaw),
    MultiPart(MultiPartRaw),
}

impl Serialize for BlockStateRaw {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BlockStateRaw::Variants(v) => {
                let list = &v.0;
                let mut s = serializer.serialize_struct("variant", list.len())?;
                
                s.end()
            }
            BlockStateRaw::MultiPart(m) => {
                let list = &m.0;
                let mut s = serializer.serialize_seq(Some(list.len()))?;
                for(k, v) in list {
                    
                    
                }
                s.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for BlockStateRaw {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        struct InnerVisitor;

        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = BlockStateRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(" multipart: {...} or variants: {...}")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                if let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "variants" => Ok(Self::Value::Variants(map.next_value()?)),
                        "multipart" => Ok(Self::Value::MultiPart(map.next_value()?)),
                        _ => Err(de::Error::unknown_field(key.as_str(), FIELDS))
                    }
                } else {
                    Err(de::Error::missing_field(""))
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["variants", "multipart"];
        deserializer.deserialize_map(InnerVisitor)
    }
}



pub fn transform(axis: Axis, rotation: Rotate90, vec3: &[f32; 3]) -> [f32; 3] {
    match rotation {
        Rotate90::R0 => {
            [vec3[0], vec3[1], vec3[2]]
        }
        Rotate90::R90 => {
            match axis {
                Axis::X => [vec3[0]         , 16.0 - vec3[2]    , vec3[1]           ],
                Axis::Y => [vec3[2]         , vec3[1]           , 16.0 - vec3[0]    ],
                Axis::Z => [16.0 - vec3[1]  , vec3[0]           , vec3[2]           ],
            } 
        }
        Rotate90::R180 => {
            match axis {
                Axis::X => [vec3[0]         , 16.0 - vec3[1]    , 16.0 - vec3[2]    ],
                Axis::Y => [16.0 - vec3[0]  , vec3[1]           , 16.0 - vec3[2]    ],
                Axis::Z => [16.0 - vec3[0]  , 16.0 - vec3[1]    , vec3[2]           ],
            } 
        }
        Rotate90::R270 => {
            match axis {
                Axis::X => [vec3[0]         , vec3[2]           , 16.0 - vec3[1]    ],
                Axis::Y => [16.0 - vec3[2]  , vec3[1]           , vec3[0]           ],
                Axis::Z => [vec3[1]         , 16.0 - vec3[0]    , vec3[2]           ],
            } 
        }
    }
}
