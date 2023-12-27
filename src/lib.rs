use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use id_arena::{Arena, Id};

pub mod compat;
pub mod pass;
pub mod utils;
pub mod adapt;

pub struct Value<T, Y, R, D> {
    pub id: Id<ValueDef<T, Y, R, D>>,
    pub idx: Option<u32>,
}

impl<T, Y, R, D> Clone for Value<T, Y, R, D> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            idx: self.idx.clone(),
        }
    }
}

pub enum Param<T, Y, R, D> {
    Value(Value<T, Y, R, D>),
    Data(Id<D>),
    Fun(Id<Fun<T, Y, R, D>>),
}
impl<T, Y, R, D> Clone for Param<T, Y, R, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Value(arg0) => Self::Value(arg0.clone()),
            Self::Data(arg0) => Self::Data(arg0.clone()),
            Self::Fun(arg0) => Self::Fun(arg0.clone()),
        }
    }
}
pub struct Params<T, Y, R, D> {
    pub values: Vec<Value<T, Y, R, D>>,
    pub data: Vec<Id<D>>,
    pub funs: Vec<Id<Fun<T, Y, R, D>>>,
}
impl<T, Y, R, D> Default for Params<T, Y, R, D> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            data: Default::default(),
            funs: Default::default(),
        }
    }
}
impl<T, Y, R, D> Clone for Params<T, Y, R, D> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            data: self.data.clone(),
            funs: self.funs.clone(),
        }
    }
}
impl<T, Y, R, D> FromIterator<Param<T, Y, R, D>> for Params<T, Y, R, D> {
    fn from_iter<I: IntoIterator<Item = Param<T, Y, R, D>>>(iter: I) -> Self {
        // let mut i = iter.into_iter();
        let mut p: Params<T, Y, R, D> = Default::default();
        for j in iter.into_iter() {
            match j {
                Param::Value(v) => p.values.push(v),
                Param::Data(d) => p.data.push(d),
                Param::Fun(f) => p.funs.push(f),
            }
        }
        return p;
    }
}
impl<T: 'static, Y: 'static, R: 'static, D: 'static> IntoIterator for Params<T, Y, R, D> {
    type Item = Param<T, Y, R, D>;

    type IntoIter = Box<dyn Iterator<Item = Param<T, Y, R, D>>>;

    fn into_iter(self) -> Self::IntoIter {
        return Box::new(
            self.values
                .into_iter()
                .map(Param::Value)
                .chain(self.data.into_iter().map(Param::Data))
                .chain(self.funs.into_iter().map(Param::Fun)),
        );
    }
}

pub enum ValueDef<T, Y, R, D> {
    Param(usize),
    Emit {
        op: T,
        params: Params<T, Y, R, D>,
        typ: Y,
        after: BTreeSet<Id<ValueDef<T, Y, R, D>>>,
    },
    Alias(Value<T, Y, R, D>),
}

impl<T: Clone, Y: Clone, R: Clone, D> Clone for ValueDef<T, Y, R, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Param(arg0) => Self::Param(arg0.clone()),
            // Self::Emit(arg0, arg1, arg2,arg3) => Self::Emit(arg0.clone(), arg1.clone(), arg2.clone(),arg3.clone()),
            Self::Alias(arg0) => Self::Alias(arg0.clone()),
            // ValueDef::Param(_) => todo!(),
            ValueDef::Emit {
                op,
                params,
                typ,
                after,
            } => ValueDef::Emit {
                op: op.clone(),
                params: params.clone(),
                typ: typ.clone(),
                after: after.clone(),
            },
            // ValueDef::Alias(_) => todo!(),
        }
    }
}

pub struct Fun<T, Y, R, D> {
    pub values: Arena<ValueDef<T, Y, R, D>>,
    pub params: usize,
    pub terminator: R,
}
impl<T, Y, R, D> Fun<T, Y, R, D> {
    pub fn param(&mut self, a: usize) -> Value<T, Y, R, D> {
        return Value {
            id: self.values.alloc(ValueDef::Param(a)),
            idx: None,
        };
    }
}
impl<T, Y, R: Default, D> Default for Fun<T, Y, R, D> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            terminator: Default::default(),
            params: Default::default(),
        }
    }
}
impl<T: Clone, Y: Clone, R: Clone, D> Clone for Fun<T, Y, R, D> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            params: self.params.clone(),
            terminator: self.terminator.clone(),
        }
    }
}

pub struct Module<T, Y, R, D> {
    pub code: Arena<Fun<T, Y, R, D>>,
    pub data: Arena<D>,
}

#[cfg(test)]
mod tests {
    use super::*;
}
