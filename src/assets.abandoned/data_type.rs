use serde::Deserialize;
use serde::Serialize;
use serde::de;
use serde::de::Deserializer;
use serde::de::Visitor;
use serde::ser::Serializer;

/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Axis {
    #[serde(rename = "x")]
    X = 0,
    #[serde(rename = "y")]
    Y = 1,
    #[serde(rename = "z")]
    Z = 2
}


/**
 * 
 */
#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub enum Face {

    #[serde(rename = "down")]
    Down, 

    #[serde(rename = "up")]
    Up, 

    #[serde(rename = "north")]
    North, 

    #[serde(rename = "south")]
    South, 

    #[serde(rename = "west")]
    West, 

    #[serde(rename = "east")]
    East,

}


impl Face {

    pub fn index(&self) -> usize {
        match self {
            Self::West => 0,
            Self::Down => 1,
            Self::North => 2,
            Self::South => 3,
            Self::Up => 4,
            Self::East => 5,
        }
    }

    pub fn opposite(&self) -> Face {
        match self {
            Self::West => Self::East,
            Self::Down => Self::Up,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::Up => Self::Down,
            Self::East => Self::West,
        }
    }
}


/**
 * 
 */
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Rotate90 {
    R0,
    R90,
    R180,
    R270,
}

impl Default for Rotate90 {
    
    fn default() -> Self {
        Rotate90::R0
    }
}

impl Rotate90 {

    pub fn index(&self) -> usize {
        match self {
            Self::R0 => 0,
            Self::R90 => 1,
            Self::R180 => 2,
            Self::R270 => 3,
        }
    }

}

const ROTATE90_ADD: [Rotate90; 7] = [Rotate90::R0,Rotate90::R90,Rotate90::R180,Rotate90::R270,Rotate90::R0,Rotate90::R90,Rotate90::R180];

impl std::ops::Add for Rotate90 {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        ROTATE90_ADD[self.index() + rhs.index()].clone()
    }
}

impl std::ops::Neg for Rotate90 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::R0 => Self::R0,
            Self::R90 => Self::R270,
            Self::R180 => Self::R180,
            Self::R270 => Self::R90,
        }
    }
}

impl<'de> Deserialize<'de> for Rotate90 {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        struct InnerVisitor;

        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = Rotate90;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("number: 0, 90, 180, 270")
            }

            fn visit_u16<E>(self, value: u16) -> Result<Rotate90, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Rotate90::R0),
                    90 => Ok(Rotate90::R90),
                    180 => Ok(Rotate90::R180),
                    270 => Ok(Rotate90::R270),
                    _ => Err(de::Error::invalid_type(de::Unexpected::Unsigned(value as u64), &self))
                }
            }

            fn visit_u32<E>(self, value: u32) -> Result<Rotate90, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Rotate90::R0),
                    90 => Ok(Rotate90::R90),
                    180 => Ok(Rotate90::R180),
                    270 => Ok(Rotate90::R270),
                    _ => Err(de::Error::invalid_type(de::Unexpected::Unsigned(value as u64), &self))
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Rotate90, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Rotate90::R0),
                    90 => Ok(Rotate90::R90),
                    180 => Ok(Rotate90::R180),
                    270 => Ok(Rotate90::R270),
                    _ => Err(de::Error::invalid_type(de::Unexpected::Unsigned(value as u64), &self))
                }
            }

            fn visit_i16<E>(self, value: i16) -> Result<Rotate90, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Rotate90::R0),
                    90 => Ok(Rotate90::R90),
                    180 => Ok(Rotate90::R180),
                    270 => Ok(Rotate90::R270),
                    _ => Err(de::Error::invalid_type(de::Unexpected::Signed(value as i64), &self))
                }
            }

            fn visit_i32<E>(self, value: i32) -> Result<Rotate90, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Rotate90::R0),
                    90 => Ok(Rotate90::R90),
                    180 => Ok(Rotate90::R180),
                    270 => Ok(Rotate90::R270),
                    _ => Err(de::Error::invalid_type(de::Unexpected::Signed(value as i64), &self))
                }
            }

            fn visit_i64<E>(self, value: i64) -> Result<Rotate90, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Rotate90::R0),
                    90 => Ok(Rotate90::R90),
                    180 => Ok(Rotate90::R180),
                    270 => Ok(Rotate90::R270),
                    _ => Err(de::Error::invalid_type(de::Unexpected::Signed(value as i64), &self))
                }
            }
        }
        deserializer.deserialize_i32(InnerVisitor)
    }
}

impl Serialize for Rotate90 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.clone() as i32)
    }
}


/**
 * 
 */

macro_rules! primary_gen_deser {
    ($($n:ident,$t:ty,$f:ident),*) => {

        pub enum PrimaryType {
            $(
            $n($t),
            )*
        }

        impl<'de> Deserialize<'de> for PrimaryType {

            fn deserialize<D>(deserializer: D) -> Result<PrimaryType, D::Error>
            where
                D: Deserializer<'de>,
            {
                use std::fmt;
                
                struct PrimaryTypeVisitor;
    
                impl<'de> Visitor<'de> for PrimaryTypeVisitor {
                    type Value = PrimaryType;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("primary types")
                    }

                    $(
                    fn $f<E>(self, value: $t) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(PrimaryType::$n(value))
                    }
                    )*

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(PrimaryType::String(value.to_string()))
                    }
                }

                deserializer.deserialize_any(PrimaryTypeVisitor)
            }
        }

        impl Into<String> for PrimaryType {
            fn into(self) -> String {
                match self {
                    $(
                    PrimaryType::$n(v) => format!("{}", v),
                    )*
                }
            }
        }
    };
}


primary_gen_deser!(
    Bool,bool,visit_bool,
    I8, i8, visit_i8,
    U8, u8, visit_u8,
    I16, i16, visit_i16,
    U16, u16, visit_u16,
    I32, i32, visit_i32,
    U32, u32, visit_u32,
    I64, i64, visit_i64,
    U64, u64, visit_u64,
    Float32, f32, visit_f32,
    Float64, f64, visit_f64,
    String,  String, visit_string   // for `&str`
);