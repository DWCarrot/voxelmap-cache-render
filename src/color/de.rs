use std::collections::btree_map::BTreeMap as Map;
use std::collections::hash_map::HashMap;
use std::io::BufRead;
use std::io::Read;
use std::io::Seek;
use std::str::FromStr;

use serde::de;
use serde::de::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Serialize;
use serde_json;

use image::DynamicImage;
use image::ImageFormat;
/**
 *
 */
use image::Rgb;
use image::Rgba;

use super::biome::BiomeColor;
use super::biome::BiomeColorTOps;
use super::biome::BiomeProps;
use super::BakedColorManager;

fn u32_to_rgb(c: u32) -> Rgb<u8> {
    Rgb::from([
        ((c >> 16) & 0xFF) as u8,
        ((c >> 8) & 0xFF) as u8,
        ((c >> 0) & 0xFF) as u8,
    ])
}

#[derive(Deserialize)]
enum BiomeColorTOpsRaw {
    Fixed(u32),
    Average(u32),
}

fn raw2ops(x: Option<BiomeColorTOpsRaw>) -> BiomeColorTOps {
    if let Some(x) = x {
        match x {
            BiomeColorTOpsRaw::Fixed(c) => BiomeColorTOps::Fixed(u32_to_rgb(c)),
            BiomeColorTOpsRaw::Average(c) => BiomeColorTOps::Average(u32_to_rgb(c)),
        }
    } else {
        BiomeColorTOps::None
    }
}

#[derive(Deserialize)]
struct BiomeTupleRaw {
    id: usize,
    name: String,
    temperature: f32,
    rainfall: f32,
    watercolor: u32,
    ops_grass: Option<BiomeColorTOpsRaw>,
    ops_foliage: Option<BiomeColorTOpsRaw>,
}

impl Into<(String, BiomeProps, Rgb<u8>, BiomeColorTOps, BiomeColorTOps)> for BiomeTupleRaw {
    fn into(self) -> (String, BiomeProps, Rgb<u8>, BiomeColorTOps, BiomeColorTOps) {
        (
            self.name,
            BiomeProps::new(self.temperature, self.rainfall),
            u32_to_rgb(self.watercolor),
            raw2ops(self.ops_grass),
            raw2ops(self.ops_foliage),
        )
    }
}

pub fn build_biomecolor<R: Read, RI: BufRead + Seek>(
    biome_data: R,
    grass_colormap: RI,
    foliage_colormap: RI,
) -> BiomeColor {
    let raws: Vec<BiomeTupleRaw> = serde_json::from_reader(biome_data).unwrap();
    let biomes = raws
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            if i == e.id {
                e.into()
            } else {
                panic!("invalid biome data: {}", e.name)
            }
        })
        .collect();
    let grass = if let DynamicImage::ImageRgb8(img) =
        image::load(grass_colormap, ImageFormat::Png).unwrap()
    {
        img
    } else {
        panic!("invalid image format: grass");
    };
    let foliage = if let DynamicImage::ImageRgb8(img) =
        image::load(foliage_colormap, ImageFormat::Png).unwrap()
    {
        img
    } else {
        panic!("invalid image format: grass");
    };
    BiomeColor::from_raw(biomes, grass, foliage)
}

/**
 *
 */
use super::blockstate::BlockState;

macro_rules! blockstate_deserialize {
    ($K:ty, $M:ty) => {
        // start code

        impl<'de> Deserialize<'de> for BlockState<$K, $M> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                use std::fmt;

                const VARIANTS: &'static [&'static str] = &["single", "variants", "multipart"];

                struct InnerVisitor;

                impl<'de> Visitor<'de> for InnerVisitor {
                    type Value = BlockState<$K, $M>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("BlockState enum")
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                    where
                        V: MapAccess<'de>,
                    {
                        while let Some(key) = map.next_key::<String>()? {
                            match key.as_str() {
                                "single" => {
                                    let v = map.next_value()?;
                                    return Ok(BlockState::build_single(v));
                                }
                                "variants" => {
                                    let VariantRaw { keys, values } = map.next_value()?;
                                    return Ok(BlockState::build_variants(keys.0, values.0));
                                }
                                _ => {
                                    let MultiPartRaw { keys, values } = map.next_value()?;
                                    let keys = keys.0;
                                    let values =
                                        values.into_iter().map(|e| (e.when, e.apply)).collect();
                                    return Ok(BlockState::build_multipart(keys, values));
                                }
                            }
                        }
                        Err(de::Error::unknown_field("", VARIANTS))
                    }
                }

                deserializer.deserialize_map(InnerVisitor)
            }
        }

        struct KMap(Map<$K, usize>);

        impl<'de> Deserialize<'de> for KMap {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                use std::fmt;

                use serde::de;

                struct InnerVisitor;

                impl<'de> Visitor<'de> for InnerVisitor {
                    type Value = KMap;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("{\"key\":number}")
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                    where
                        V: MapAccess<'de>,
                    {
                        let mut result = Map::new();
                        while let Some(sc) = map.next_key::<String>()? {
                            let s = sc.as_str();
                            let key = <$K>::from_str(s).map_err(|e| {
                                de::Error::custom(format!("unable to parse key: {}", s))
                            })?;
                            let value = map.next_value()?;
                            result.insert(key, value);
                        }
                        Ok(KMap(result))
                    }
                }

                deserializer.deserialize_map(InnerVisitor)
            }
        }

        struct NMap(Map<usize, $M>);

        impl<'de> Deserialize<'de> for NMap {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                use std::fmt;

                use serde::de;

                struct InnerVisitor;

                impl<'de> Visitor<'de> for InnerVisitor {
                    type Value = NMap;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("{\"number\":value}")
                    }

                    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                    where
                        V: MapAccess<'de>,
                    {
                        let mut result = Map::new();
                        while let Some(sc) = map.next_key::<String>()? {
                            let s = sc.as_str();
                            let key = usize::from_str(s).map_err(|e| {
                                de::Error::custom(format!("unable to parse key: {}", s))
                            })?;
                            let value = map.next_value()?;
                            result.insert(key, value);
                        }
                        Ok(NMap(result))
                    }
                }

                deserializer.deserialize_map(InnerVisitor)
            }
        }

        #[derive(Deserialize)]
        struct VariantRaw {
            keys: KMap,
            values: NMap,
        }

        #[derive(Deserialize)]
        struct MultiPartElementRaw {
            when: Vec<usize>,
            apply: $M,
        }

        #[derive(Deserialize)]
        struct MultiPartRaw {
            keys: KMap,
            values: Vec<MultiPartElementRaw>,
        }

        // end code
    };
}

/**
 *
 */
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

impl FromStr for KVPair {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i = s
            .find('=')
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
        let key = String::from(&s[..i]);
        let val = String::from(&s[i + 1..]);
        Ok(KVPair { key, val })
    }
}

pub type BlockStateC = BlockState<String, usize>;

blockstate_deserialize!(String, usize);

#[derive(Deserialize)]
struct IndexRaw {
    data: HashMap<String, BlockStateC>,
}

pub fn build_backedcolormanager<R: Read, RI: BufRead + Seek>(
    index_file: R,
    colormap_file: RI,
    weightmap_file: RI,
    biome_color: BiomeColor,
) -> BakedColorManager {
    let json: IndexRaw = serde_json::from_reader(index_file).unwrap();
    let colormap = if let DynamicImage::ImageRgba8(img) =
        image::load(colormap_file, ImageFormat::Png).unwrap()
    {
        img
    } else {
        panic!("invalid image: colormap");
    };
    let weightmap = if let DynamicImage::ImageLuma8(img) =
        image::load(weightmap_file, ImageFormat::Png).unwrap()
    {
        img
    } else {
        panic!("invalid image: weightmap");
    };
    BakedColorManager::from_raw(json.data, colormap, weightmap, biome_color)
}
