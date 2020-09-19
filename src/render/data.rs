pub const TILESIZE: (u32, u32) = (256, 256);

pub struct LayerView<'a> {
    raw: &'a[u8],
}

impl<'a> LayerView<'a> {

    pub fn height(&self) -> u8 {
        self.raw[0]
    }

    pub fn blockstate_id(&self) -> u16 {
        ((self.raw[1] as u16) << 8) | (self.raw[2] as u16) 
    }

    pub fn light(&self) -> u8 {
        self.raw[3]
    }

    pub fn skylight(&self) -> u8 {
        (self.raw[3] & 0xF0) >> 4
    }

    pub fn blocklight(&self) -> u8 {
        (self.raw[3] & 0x0F) >> 0
    }
}

pub struct ElementView<'a> {
    raw: &'a[u8],
}

impl<'a> ElementView<'a> {

    pub fn surface(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[0..4] }
    }

    pub fn seafloor(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[4..8] }
    }

    pub fn transparent(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[8..12] }
    }

    pub fn foliage(&self) -> LayerView<'a> {
        LayerView { raw: &self.raw[12..16] }
    }

    // pub fn _(&self) -> u8 {
    //     self.raw[16]
    // }

    pub fn biome(&self) -> u8 {
        self.raw[17]
    }
}

pub struct TileView<'a> {
    raw: &'a[u8],
}

impl<'a> TileView<'a> {

    pub fn bind(raw: &'a[u8]) -> Self {
        TileView { raw }
    }

    pub fn element(&self, x: u32, z: u32) -> ElementView<'a> {
        let index = ((x + z * TILESIZE.0) * 18) as usize;
        ElementView { raw: &self.raw[index .. index + 18] }
    }
}