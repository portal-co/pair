use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use id_arena::Arena;

use crate::compat::*;

pub struct FuncTransformCtx<A: ModLike, B: ModLike> {
    pub input: <A::Code as ArenaLike<A::Fun>>::Id,
    pub output: <B::Code as ArenaLike<B::Fun>>::Id,
}

pub struct PassState<'a, 'b, A: ModLike, B: ModLike> {
    pub input: &'a A,
    pub out: &'b mut B,
    pub code_cache:
        BTreeMap<<A::Code as ArenaLike<A::Fun>>::Id, <B::Code as ArenaLike<B::Fun>>::Id>,
    pub datum_cache:
        BTreeMap<<A::Data as ArenaLike<A::Datum>>::Id, <B::Data as ArenaLike<B::Datum>>::Id>,
}
macro_rules! emitter {
    ($ty:tt => $trait:tt, $a:tt, $b:tt, $e:tt,$p:tt,$c:lifetime,$d:lifetime) => {
        pub trait $trait<$c, $d, $a: ModLike, $b: ModLike, $e, $p: PassBehavior<$a, $b, $e>>:
            $ty
        {
        }
        impl<$c, $d, $a: ModLike, $b: ModLike, $e, $p: PassBehavior<$a, $b, $e>, T: $ty>
            $trait<$c, $d, $a, $b, $e, $p> for T
        {
        }
    };
}
emitter!((FnMut(&mut P, &mut PassState<'a,'b,A,B>, ValID<A>) -> Result<ValID<B>,E>) => ValEmit, A, B,E, P,'a,'b);
emitter!((FnMut(&mut P, &mut PassState<'a,'b,A,B>, FunId<A>) -> Result<FunId<B>,E>) => FunEmit, A, B,E, P,'a,'b);
emitter!((FnMut(&mut P, &mut PassState<'a,'b,A,B>, DatId<A>) -> Result<DatId<B>,E>) => DatEmit, A, B,E, P,'a,'b);
pub trait PassBehavior<A: ModLike, B: ModLike, Err>: Sized {
    fn value<'a, 'b>(
        &mut self,
        ctx: &mut PassState<'a, 'b, A, B>,
        fun_ctx: FuncTransformCtx<A, B>,
        it: <A::Fun as FunLike>::Value,
        value: impl ValEmit<'a, 'b, A, B, Err, Self>,
        fun: impl FunEmit<'a, 'b, A, B, Err, Self>,
        dat: impl DatEmit<'a, 'b, A, B, Err, Self>,
    ) -> Result<<B::Fun as FunLike>::Value, Err>;
    fn terminator<'a, 'b>(
        &mut self,
        ctx: &mut PassState<A, B>,
        fun_ctx: FuncTransformCtx<A, B>,
        it: <A::Fun as FunLike>::Terminator,
        value: impl ValEmit<'a, 'b, A, B, Err, Self>,
        fun: impl FunEmit<'a, 'b, A, B, Err, Self>,
        dat: impl DatEmit<'a, 'b, A, B, Err, Self>,
    ) -> Result<<B::Fun as FunLike>::Terminator, Err>;
    fn datum<'a, 'b>(
        &mut self,
        ctx: &mut PassState<A, B>,
        def: A::Datum,
        fun: impl FunEmit<'a, 'b, A, B, Err, Self>,
        dat: impl DatEmit<'a, 'b, A, B, Err, Self>,
    ) -> Result<B::Datum, Err>;
}
pub type ValueTransMap<A: ModLike, B: ModLike> = Rc<RefCell<BTreeMap<ValID<A>, ValID<B>>>>;
impl<'a, 'b, A: ModLike, B: ModLike> PassState<'a, 'b, A, B>
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
    pub fn func_value<E>(
        &mut self,
        w: &mut impl PassBehavior<A, B, E>,
        f: FunId<A>,
        m: ValueTransMap<A, B>,
        a: ValID<A>,
    ) -> Result<ValID<B>, E> {
        {
            if let Some(x) = m.borrow_mut().get(&a) {
                return Ok(x.clone());
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
        }?;
        self.out.code_mut()[self.code_cache.get(&f).unwrap().clone()].all_mut()[i.clone()] = v;
        return Ok(i);
    }
    pub fn func<Err>(
        &mut self,
        w: &mut impl PassBehavior<A, B, Err>,
        f: FunId<A>,
    ) -> Result<FunId<B>, Err> {
        if let Some(i) = self.code_cache.get(&f) {
            return Ok(i.clone());
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
        )?;
        *self.out.code_mut()[me.clone()].terminator_mut() = t;
        return Ok(me);
        // });
    }
    pub fn dat<E>(
        &mut self,
        w: &mut impl PassBehavior<A, B, E>,
        dd: DatId<A>,
    ) -> Result<DatId<B>, E> {
        if let Some(b) = self.datum_cache.get(&dd) {
            return Ok(b.clone());
        }
        let d = w.datum(
            self,
            self.input.data()[dd.clone()].clone(),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        )?;
        let e = self.out.data_mut().push(d);
        self.datum_cache.insert(dd, e.clone());
        return Ok(e);
    }
}
