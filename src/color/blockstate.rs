use std::collections::btree_map::BTreeMap as Map;
use std::collections::btree_map::Values;
use std::collections::btree_map::ValuesMut;
use std::borrow::Borrow;


/**
 * 
 */
#[derive(Clone, Debug)]
pub struct Expression<K, V, M> {

    keys: Map<K, M>,

    values: Map<M, V>,

    mask: M,
}

impl<K: std::cmp::Ord, V> Default for Expression<K, V, usize> {

    fn default() -> Self {
        Expression {
            keys: Map::default(),
            values: Map::default(),
            mask: 0,
        }
    }
}

impl<K: std::cmp::Ord + Clone, V> Expression<K, V, usize> {

    pub fn insert<'a, I:Iterator<Item=&'a K>>(&'a mut self, key: I, value: V) -> Option<usize> {
        let mut m = self.get_initial_link();
        for k in key {
            self.insert_key(k, &mut m)?;
        }
        self.values.insert(m, value);
        Some(m)
    }

    pub fn insert_key(&mut self, k: &K, m: &mut usize) -> Option<usize> {
        if let Some(n) = self.keys.get(k) {
            *m |= *n;
            Some(*n)
        } else {
            if self.mask == std::usize::MAX {
                return None;
            }
            let n = self.mask + 1;
            self.mask = n | self.mask;
            self.keys.insert(k.clone(), n);
            *m |= n;
            Some(n)
        }
    }

    pub fn insert_value_unchecked(&mut self, m: usize, v: V) -> Option<V> {
        self.values.insert(m, v)
    }
}

impl<K: std::cmp::Ord, V> Expression<K, V, usize> {

    pub fn get_initial_link(&self) -> usize {
        0
    }

    pub fn size(&self) -> (usize, usize) {
        (self.keys.len(), self.values.len())
    }

    pub fn get<'a, Q: 'a, I>(&'a self, key: I, strict: bool) -> Option<&'a V> 
    where
        Q: std::cmp::Ord,
        K: Borrow<Q>,
        I: Iterator<Item = &'a Q>,
    {
        if !strict && self.keys.len() == 0 {
            return self.values.get(&0);
        }
        let mut m = 0;
        let keys = &self.keys;
        for k in key {
            match keys.get(k) {
                Some(n) => {
                    m |= n;
                },
                None => {
                    if strict { return None; }
                }
            }
        }
        self.values.get(&m)
    }

    pub fn get_mut<'a, Q: 'a, I>(&'a mut self, key: I, strict: bool) -> Option<&'a mut V> 
    where
        Q: std::cmp::Ord,
        K: Borrow<Q>,
        I: Iterator<Item = &'a Q>,
    {
        if !strict && self.keys.len() == 0 {
            return self.values.get_mut(&0);
        }
        let mut m = 0;
        let keys = &self.keys;
        for k in key {
            match keys.get(k) {
                Some(n) => {
                    m |= n;
                },
                None => {
                    if strict { return None; }
                }
            }
        }
        self.values.get_mut(&m)
    }

    pub fn all<'a>(&'a self) -> Values<'a, usize, V> {
        self.values.values()
    }

    pub fn all_mut<'a>(&'a mut self) -> ValuesMut<'a, usize, V> {
        self.values.values_mut()
    }

    pub fn transf_into<K2, V2, FK, FV, E>(self, mut fk: FK, mut fv: FV) -> Result<Expression<K2, V2, usize>, E> 
    where
        K2: std::cmp::Ord,
        FK: FnMut(K)->Result<K2, E>, 
        FV: FnMut(V)->Result<V2, E>,
    {
        let mut keys = Map::new();
        for (k, m) in self.keys {
            let k2 = fk(k)?;
            keys.insert(k2, m);
        }
        let mut values = Map::new();
        for (m, v) in self.values {
            let v2 = fv(v)?;
            values.insert(m, v2);
        }
        Ok(Expression {
            keys,
            values,
            mask: self.mask
        })
    }
}


/**
 * 
 */

#[derive(Clone, Debug)]
pub enum BlockState<K, M> {
    Single(M),
    Variants(Expression<K, M, usize>),
    MultiPart(Expression<K, M, usize>, Vec<usize>),
}

impl<K: std::cmp::Ord, M: Clone> BlockState<K, M> {

    pub fn build_single(m: M) -> Self {
        Self::Single(m)
    }

    pub fn build_variants(keys: Map<K, usize>, values: Map<usize, M>) -> Self {
        let mut mask = 0;
        for (k, v) in &keys {
            mask |= v;
        }
        Self::Variants(Expression { keys, values, mask})
    }

    pub fn build_multipart(keys: Map<K, usize>, values: Vec<(Vec<usize>, M)>) -> Self {
        let mut mask = 0;
        for (k, v) in &keys {
            mask |= v;
        }
        let mut masks = Vec::with_capacity(values.len());
        let mut mapvalues = Map::new();
        for (when, val) in values {
            let mut groupmask = 0;
            for k in when {
                mapvalues.insert(k, val.clone());
                groupmask |= k;
            }
            masks.push(groupmask);
        }
        Self::MultiPart(Expression {keys, values: mapvalues, mask},  masks)
    }

    pub fn get<'a, Q: 'a + ?Sized, I>(&'a self, key: I) -> Vec<M> 
    where
        Q: std::cmp::Ord,
        K: Borrow<Q>,
        I: Iterator<Item = &'a Q>,
    {
        match self {
            Self::Single(model) => {
                vec![model.clone()]
            }
            Self::Variants(expr) => {
                let mut m = 0;
                for k in key {
                    if let Some(n) = expr.keys.get(k) {
                        m |= n;
                    }
                }
                if let Some(model) = expr.values.get(&m) {
                    vec![model.clone()]
                } else {
                    Vec::new()
                }
            }
            Self::MultiPart(expr, group) => {
                let mut m = 0;
                for k in key {
                    if let Some(n) = expr.keys.get(k) {
                        m |= n;
                    }
                }
                let mut parts = Vec::with_capacity(group.len());
                for mask in group.iter() {
                    let n = *mask & m;
                    if let Some(model) = expr.values.get(&n) {
                        parts.push(model.clone())
                    }
                }
                parts
            }
        }
        
    }
} 

impl<K: std::cmp::Ord + Clone, M> BlockState<K, M> {

    pub fn start_group(&mut self) {
        match self {
            Self::MultiPart(expr, group) => {
                group.push(expr.get_initial_link());
            },
            _ => {
                panic!("invalid operation")
            }
        }
    }

    pub fn insert_group<'a, I:Iterator<Item=&'a K>>(&'a mut self, key: I, value: M) -> Option<usize> {
        match self {
            Self::MultiPart(expr, group) => {
                let mut m = expr.get_initial_link();
                for k in key {
                    expr.insert_key(k, &mut m)?;
                }
                let mask = group.last_mut()?;
                expr.values.insert(m, value);
                *mask |= m;
                Some(m)
            },
            Self::Variants(expr) => {
                let mut m = expr.get_initial_link();
                for k in key {
                    expr.insert_key(k, &mut m)?;
                }
                expr.values.insert(m, value);
                Some(m)
            }
            Self::Single(model) => {
                panic!("invalid operation")
            }
        }
    }

    pub fn try_simplify_variant(self) -> Self {
        match self {
            Self::Variants(mut expr) => {
                if expr.values.len() == 1 {
                    if let Some(model) = expr.values.remove(&0) {
                        return Self::Single(model)
                    }
                }
                Self::Variants(expr)
            },
            _ => {
                panic!("invalid operation")
            }
        }
    }
}




