use std::marker::PhantomData;

pub const TILESIZE: (u32, u32) = (256, 256);

pub trait View<'a> {
    type EN;
    type LN;

    fn element(&self, x: u32, z: u32) -> Self::EN;

    fn surface(&self, element: Self::EN) -> Self::LN;
    
    fn seafloor(&self, element: Self::EN) -> Self::LN;

    fn transparent(&self, element: Self::EN) -> Self::LN;

    fn foliage(&self, element: Self::EN) -> Self::LN;

    fn biome(&self, element: Self::EN) -> u16;

    fn height(&self, layer: Self::LN) -> u8;

    fn blockstate_id(&self, layer: Self::LN) -> u16;

    fn light(&self, layer: Self::LN) -> u8;

    fn skylight(&self, layer: Self::LN) -> u8;

    fn blocklight(&self, layer: Self::LN) -> u8;
}


#[derive(Clone, Copy)]
pub struct LayerNode<'a> {
    ptr: *const u8,
    _life: PhantomData<&'a [u8]>
}

impl<'a> LayerNode<'a> {

    fn new(element: ElementNode<'a>, offset: usize) -> LayerNode<'a> {
        LayerNode {
            ptr: element.ptr.wrapping_offset(offset as isize),
            _life: element._life
        }
    }

    unsafe fn get(&self, offset: usize) -> u8 {
        *(self.ptr.wrapping_offset(offset as isize))
    }
}


#[derive(Clone, Copy)]
pub struct ElementNode<'a> {
    ptr: *const u8,
    _life: PhantomData<&'a [u8]>
}

impl<'a> ElementNode<'a> {

    fn new(raw: &'a [u8], offset: usize) -> ElementNode<'a> {
        ElementNode {
            ptr: raw.as_ptr().wrapping_offset(offset as isize),
            _life: PhantomData
        }
    }

    unsafe fn get(&self, offset: usize) -> u8 {
        *(self.ptr.wrapping_offset(offset as isize))
    }
}


/**
 * 
 */

pub struct V1TileView<'a> {
    raw: &'a[u8],
}

impl<'a> View<'a> for V1TileView<'a> {
    type LN = LayerNode<'a>;
    type EN = ElementNode<'a>;

    fn element(&self, x: u32, z: u32) -> Self::EN {
        let offset = ((x + z * TILESIZE.0) * 18) as usize;
        ElementNode::new(self.raw, offset)
    }

    fn surface(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 0)
    }
    
    fn seafloor(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 4)
    }

    fn transparent(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 8)
    }

    fn foliage(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 12)
    }

    fn biome(&self, element: Self::EN) -> u16 {
        unsafe {
            ((element.get(16) as u16) << 8) | (element.get(17) as u16)
        }
    }

    fn height(&self, layer: Self::LN) -> u8 {
        unsafe {
            layer.get(0)
        }
    }

    fn blockstate_id(&self, layer: Self::LN) -> u16 {
        unsafe {
            ((layer.get(1) as u16) << 8) | (layer.get(2) as u16)
        }      
    }

    fn light(&self, layer: Self::LN) -> u8 {
        unsafe {
            layer.get(3)
        }
    }

    fn skylight(&self, layer: Self::LN) -> u8 {
        (self.light(layer) & 0xF0) >> 4
    }

    fn blocklight(&self, layer: Self::LN) -> u8 {
        self.light(layer) & 0x0F
    }
}

impl<'a> V1TileView<'a> {

    pub fn bind(raw: &'a [u8]) -> V1TileView<'a> {
        let sz = (TILESIZE.0 * TILESIZE.1 * 18) as usize;
        if raw.len() < sz {
            panic!("index out of bounds: {} > {}", sz - 1, raw.len());
        }
        V1TileView {
            raw
        }
    }
}


/**
 * 
 */

pub struct V2TileView<'a> {
    raw: &'a[u8],
}

impl<'a> View<'a> for V2TileView<'a> {
    type LN = LayerNode<'a>;
    type EN = ElementNode<'a>;

    fn element(&self, x: u32, z: u32) -> Self::EN {
        let offset = ((x + z * TILESIZE.0) * 1) as usize;
        ElementNode::new(self.raw, offset)
    }

    fn surface(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 0 * (TILESIZE.1 * TILESIZE.0) as usize)
    }
    
    fn seafloor(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 4 * (TILESIZE.1 * TILESIZE.0) as usize)
    }

    fn transparent(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 8 * (TILESIZE.1 * TILESIZE.0) as usize)
    }

    fn foliage(&self, element: Self::EN) -> Self::LN {
        LayerNode::new(element, 12 * (TILESIZE.1 * TILESIZE.0) as usize)
    }

    fn biome(&self, element: Self::EN) -> u16 {
        let step = (TILESIZE.1 * TILESIZE.0) as usize;
        unsafe {
            ((element.get(16 * step) as u16) << 8) | (element.get(17 * step) as u16)
        }
    }

    fn height(&self, layer: Self::LN) -> u8 {
        unsafe {
            layer.get(0 * (TILESIZE.1 * TILESIZE.0) as usize)
        }
    }

    fn blockstate_id(&self, layer: Self::LN) -> u16 {
        let step = (TILESIZE.1 * TILESIZE.0) as usize;
        unsafe {
            ((layer.get(1 * step) as u16) << 8) | (layer.get(2 * step) as u16)
        }      
    }

    fn light(&self, layer: Self::LN) -> u8 {
        unsafe {
            layer.get(3 * (TILESIZE.1 * TILESIZE.0) as usize)
        }
    }

    fn skylight(&self, layer: Self::LN) -> u8 {
        (self.light(layer) & 0xF0) >> 4
    }

    fn blocklight(&self, layer: Self::LN) -> u8 {
        self.light(layer) & 0x0F
    }
}

impl<'a> V2TileView<'a> {

    pub fn bind(raw: &'a [u8]) -> V2TileView<'a> {
        let sz = (TILESIZE.0 * TILESIZE.1 * 18) as usize;
        if raw.len() < sz {
            panic!("index out of bounds: {} > {}", sz - 1, raw.len());
        }
        V2TileView {
            raw
        }
    }
}