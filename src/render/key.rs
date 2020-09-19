use std::str::Split;
use std::convert::TryFrom;

use crate::color::biome::InnerColor;


pub struct BlockProps {

    pub air: bool,

    pub water: bool,

    pub waterlogged: bool,

    pub biome_color: InnerColor,
}

impl BlockProps {

    pub fn new() -> Self {
        BlockProps {
            air: true,
            water: false,
            waterlogged: false,
            biome_color: InnerColor::None,
        }
    }

    pub fn new_from<'a, I: Iterator<Item = &'a str>>(name: &'a str, state: I) -> Self {
        let mut waterlogged = false;
        for s in state {
            let mut it = s.split('=');
            if it.next() == Some("waterlogged") {
                if it.next() == Some("true") {
                    waterlogged = true;
                }
            }
        }
        BlockProps {
            air: name == "minecraft:air",
            water: name == "minecraft:water",
            waterlogged,
            biome_color: InnerColor::from(name)
        }
    }
}

#[derive(Debug)]
pub struct KeyLine<'a> {
    pub id: usize,
    pub name: &'a str,
    pub state: Option<&'a str>,
}

impl<'a> TryFrom<&'a str> for KeyLine<'a> {
    type Error = usize;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut state = None;

        let mut p = value;
        let mut pos = 0;

        let i = p.find(' ').ok_or(pos)?;
        let id = p[0..i].parse().map_err(|e| pos)?;
        p = &p[i+1..];
        pos += i + 1;

        let i = p.find('{').ok_or(pos)?;
        let s = &p[0..i];
        if s != "Block" {
            return Err(pos);
        }
        p = &p[i+1..];
        pos += i + 1;

        let i = p.find('}').ok_or(pos)?;
        let name = &p[0..i];
        p = &p[i+1..];
        pos += i + 1;

        if p.starts_with('[') {
            p = &p[1..];
            pos += 1;
            let i = p.find(']').ok_or(pos)?;
            state = Some(&p[0..i]);
        }

        Ok(KeyLine {
            id,
            name,
            state
        })
    }
}


pub struct SplitIter<'a>(Option<Split<'a, char>>);

impl<'a> From<Option<&'a str>> for SplitIter<'a> {

    fn from(value: Option<&'a str>) -> Self {
        SplitIter(value.map(|s| s.split(',')))
    }
}

impl<'a> Iterator for SplitIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(it) = &mut self.0 {
            it.next()
        } else {
            None
        }
    }
}