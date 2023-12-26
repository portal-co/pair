use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use id_arena::{Arena, Id};

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

pub struct ModuleTransform<T, Y, R, D, S, L, E, P> {
    pub input: Module<T, Y, R, D>,
    pub out: Module<S, L, E, P>,
    pub datum_cache: BTreeMap<Id<D>, Id<P>>,
    pub code_cache: BTreeMap<Id<Fun<T, Y, R, D>>, Id<Fun<S, L, E, P>>>,
}
pub struct FuncTransformCtx<T, Y, R, D, S, L, E, P> {
    pub input: Id<Fun<T, Y, R, D>>,
    pub output: Id<Fun<S, L, E, P>>,
}
pub trait MTBehavior<T, Y, R, D, S, L, E, P> {
    fn value(
        &mut self,
        ctx: &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
        fun_ctx: FuncTransformCtx<T, Y, R, D, S, L, E, P>,
        def: ValueDef<T, Y, R, D>,
        value: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<ValueDef<T, Y, R, D>>,
        ) -> Id<ValueDef<S, L, E, P>>,
        func: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<Fun<T, Y, R, D>>,
        ) -> Id<Fun<S, L, E, P>>,
        dat: impl FnMut(&mut Self, &mut ModuleTransform<T, Y, R, D, S, L, E, P>, Id<D>) -> Id<P>,
    ) -> ValueDef<S, L, E, P>;
    fn terminator(
        &mut self,
        ctx: &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
        fun_ctx: FuncTransformCtx<T, Y, R, D, S, L, E, P>,
        def: R,
        value: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<ValueDef<T, Y, R, D>>,
        ) -> Id<ValueDef<S, L, E, P>>,
        func: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<Fun<T, Y, R, D>>,
        ) -> Id<Fun<S, L, E, P>>,
        dat: impl FnMut(&mut Self, &mut ModuleTransform<T, Y, R, D, S, L, E, P>, Id<D>) -> Id<P>,
    ) -> E;
    fn datum(
        &mut self,
        ctx: &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
        def: D,
        func: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<Fun<T, Y, R, D>>,
        ) -> Id<Fun<S, L, E, P>>,
        dat: impl FnMut(&mut Self, &mut ModuleTransform<T, Y, R, D, S, L, E, P>, Id<D>) -> Id<P>,
    ) -> P;
}
pub type ValueTransMap<T, Y, R, D, S, L, E, P> =
    Rc<RefCell<BTreeMap<Id<ValueDef<T, Y, R, D>>, Id<ValueDef<S, L, E, P>>>>>;
impl<T: Clone, Y: Clone, R: Clone, D: Clone, S, L, E: Default, P>
    ModuleTransform<T, Y, R, D, S, L, E, P>
{
    pub fn func_value<W: MTBehavior<T, Y, R, D, S, L, E, P>>(
        &mut self,
        w: &mut W,
        f: Id<Fun<T, Y, R, D>>,
        m: ValueTransMap<T, Y, R, D, S, L, E, P>,
        a: Id<ValueDef<T, Y, R, D>>,
    ) -> Id<ValueDef<S, L, E, P>> {
        {
            if let Some(x) = m.borrow_mut().get(&a) {
                return *x;
            }
        }
        let i = self.out.code[*self.code_cache.get(&f).unwrap()]
            .values
            .alloc(ValueDef::Param(usize::MAX));
        let v = {
            {
                m.borrow_mut().insert(a.clone(), i.clone())
            };
            w.value(
                self,
                FuncTransformCtx {
                    input: f.clone(),
                    output: *self.code_cache.get(&f).unwrap(),
                },
                self.input.code[f].values[a].clone(),
                |w, t, v| t.func_value(w, f, m.clone(), v),
                |w, t, f| t.func(w, f),
                |w, t, d| t.dat(w, d),
            )
        };
        self.out.code[*self.code_cache.get(&f).unwrap()].values[i] = v;
        return i;
    }
    pub fn func<W: MTBehavior<T, Y, R, D, S, L, E, P>>(
        &mut self,
        w: &mut W,
        f: Id<Fun<T, Y, R, D>>,
    ) -> Id<Fun<S, L, E, P>> {
        if let Some(i) = self.code_cache.get(&f) {
            return *i;
        }
        let me = self.out.code.alloc(Default::default());
        let m = Rc::new(RefCell::new(BTreeMap::new()));
        // return self.out.code.alloc_with_id(|me| {
        self.code_cache.insert(f.clone(), me.clone());
        let a = &self.input.code[f.clone()];
        let t = w.terminator(
            self,
            FuncTransformCtx {
                input: f.clone(),
                output: me.clone(),
            },
            self.input.code[f].terminator.clone(),
            |w, t, v| t.func_value(w, f, m.clone(), v),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        );
        self.out.code[me].terminator = t;
        return me;
        // });
    }
    pub fn dat<W: MTBehavior<T, Y, R, D, S, L, E, P>>(&mut self, w: &mut W, dd: Id<D>) -> Id<P> {
        if let Some(b) = self.datum_cache.get(&dd) {
            return *b;
        }
        let d = w.datum(
            self,
            self.input.data[dd].clone(),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        );
        let e = self.out.data.alloc(d);
        self.datum_cache.insert(dd, e);
        return e;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
}
