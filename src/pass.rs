use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use id_arena::Arena;

use crate::compat::*;

pub struct FuncTransformCtx<A: ModLike, B: ModLike> {
    pub input: <A::Code as ArenaLike<A::Fun>>::Id,
    pub output: <B::Code as ArenaLike<B::Fun>>::Id,
}

pub struct PassState<A: ModLike, B: ModLike> {
    pub input: A,
    pub out: B,
    pub code_cache:
        BTreeMap<<A::Code as ArenaLike<A::Fun>>::Id, <B::Code as ArenaLike<B::Fun>>::Id>,
    pub datum_cache:
        BTreeMap<<A::Data as ArenaLike<A::Datum>>::Id, <B::Data as ArenaLike<B::Datum>>::Id>,
}
macro_rules! emitter {
    ($ty:tt => $trait:tt, $a:tt, $b:tt, $p:tt) => {
        pub trait $trait<$a: ModLike, $b: ModLike, $p: PassBehavior<$a, $b>>: $ty {}
        impl<$a: ModLike, $b: ModLike, $p: PassBehavior<$a, $b>, T: $ty> $trait<$a, $b, $p> for T {}
    };
}
emitter!((FnMut(&mut P, &mut PassState<A,B>, ValID<A>) -> ValID<B>) => ValEmit, A, B, P);
emitter!((FnMut(&mut P, &mut PassState<A,B>, FunId<A>) -> FunId<B>) => FunEmit, A, B, P);
emitter!((FnMut(&mut P, &mut PassState<A,B>, DatId<A>) -> DatId<B>) => DatEmit, A, B, P);
pub trait PassBehavior<A: ModLike, B: ModLike>: Sized {
    fn value(
        &mut self,
        ctx: &mut PassState<A, B>,
        fun_ctx: FuncTransformCtx<A, B>,
        it: <A::Fun as FunLike>::Value,
        value: impl ValEmit<A, B, Self>,
        fun: impl FunEmit<A, B, Self>,
        dat: impl DatEmit<A, B, Self>,
    ) -> <B::Fun as FunLike>::Value;
    fn terminator(
        &mut self,
        ctx: &mut PassState<A, B>,
        fun_ctx: FuncTransformCtx<A, B>,
        it: <A::Fun as FunLike>::Terminator,
        value: impl ValEmit<A, B, Self>,
        fun: impl FunEmit<A, B, Self>,
        dat: impl DatEmit<A, B, Self>,
    ) -> <B::Fun as FunLike>::Terminator;
    fn datum(
        &mut self,
        ctx: &mut PassState<A, B>,
        def: A::Datum,
        fun: impl FunEmit<A, B, Self>,
        dat: impl DatEmit<A, B, Self>,
    ) -> B::Datum;
}
pub type ValueTransMap<A: ModLike, B: ModLike> = Rc<RefCell<BTreeMap<ValID<A>, ValID<B>>>>;
impl<A: ModLike, B: ModLike> PassState<A, B>
where
    ValID<A>: Eq + Ord + Clone,
    ValID<B>: Eq + Ord + Clone,
    FunId<A>: Eq + Ord + Clone,
    FunId<B>: Eq + Ord + Clone,
    DatId<A>: Eq + Ord + Clone,
    DatId<B>: Eq + Ord + Clone,
    <A::Fun as FunLike>::Value: Clone,
    <B::Fun as FunLike>::Value: Default,
    B::Fun: Default,
    <A::Fun as FunLike>::Terminator: Clone,
    A::Datum: Clone,
{
    pub fn func_value(
        &mut self,
        w: &mut impl PassBehavior<A, B>,
        f: FunId<A>,
        m: ValueTransMap<A, B>,
        a: ValID<A>,
    ) -> ValID<B> {
        {
            if let Some(x) = m.borrow_mut().get(&a) {
                return x.clone();
            }
        }
        let i = self.out.code_mut()[self.code_cache.get(&f).unwrap().clone()]
            .all_mut()
            .push(Default::default());
        let v = {
            {
                m.borrow_mut().insert(a.clone(), i.clone())
            };
            let f2 = f.clone();
            w.value(
                self,
                FuncTransformCtx {
                    input: f.clone(),
                    output: self.code_cache.get(&f).unwrap().clone(),
                },
                self.input.code()[f.clone()].all()[a].clone(),
                |w, t, v| t.func_value(w, f2.clone(), m.clone(), v),
                |w, t, f| t.func(w, f),
                |w, t, d| t.dat(w, d),
            )
        };
        self.out.code_mut()[self.code_cache.get(&f).unwrap().clone()].all_mut()[i.clone()] = v;
        return i;
    }
    pub fn func(&mut self, w: &mut impl PassBehavior<A, B>, f: FunId<A>) -> FunId<B> {
        if let Some(i) = self.code_cache.get(&f) {
            return i.clone();
        }
        let me = self.out.code_mut().push(Default::default());
        let m = Rc::new(RefCell::new(BTreeMap::new()));
        // return self.out.code.alloc_with_id(|me| {
        self.code_cache.insert(f.clone(), me.clone());
        // let a = &self.input.code()[f.clone()];
        let f2 = f.clone();
        let t = w.terminator(
            self,
            FuncTransformCtx {
                input: f.clone(),
                output: me.clone(),
            },
            self.input.code()[f].terminator().clone(),
            |w, t, v| t.func_value(w, f2.clone(), m.clone(), v),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        );
        *self.out.code_mut()[me.clone()].terminator_mut() = t;
        return me;
        // });
    }
    pub fn dat(&mut self, w: &mut impl PassBehavior<A, B>, dd: DatId<A>) -> DatId<B> {
        if let Some(b) = self.datum_cache.get(&dd) {
            return b.clone();
        }
        let d = w.datum(
            self,
            self.input.data()[dd.clone()].clone(),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        );
        let e = self.out.data_mut().push(d);
        self.datum_cache.insert(dd, e.clone());
        return e;
    }
}
