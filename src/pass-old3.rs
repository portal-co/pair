use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use crate::compat::{ArenaLike, DatId, FunId, FunLike, ModLike, Term, Val, ValID, ValIDFun};
pub struct FunRef<Other: ModLike> {
    r#ref: *mut Other,
    pub fun: FunId<Other>,
}
impl<Other: ModLike> FunRef<Other>
where
    FunId<Other>: Clone,
{
    pub fn other(&self) -> &Other {
        return unsafe { &*self.r#ref };
    }
    pub fn other_mut(&mut self) -> &mut Other {
        return unsafe { &mut *self.r#ref };
    }
    pub fn fun(&self) -> &Other::Fun {
        return &(self.other().code()[self.fun.clone()]);
    }
    pub fn fun_mut(&mut self) -> &mut Other::Fun {
        let f = self.fun.clone();
        return &mut (self.other_mut().code_mut()[f]);
    }
}
pub trait CodeCache<In: ModLike, T> {
    fn cache<E>(
        &mut self,
        val: ValID<In>,
        go: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, E>;
}
pub trait ModCodeCache<In: ModLike, T, U> {
    fn cache(&mut self, fun: FunId<In>, go: impl FnOnce(&mut Self) -> T) -> T;
    fn cache_datum(&mut self, dat: DatId<In>, go: impl FnOnce(&mut Self) -> U) -> U;
}
pub trait PassVal<
    K,
    E,
    Other: ModLike,
    Fun: PassFun<K, E, Other, In, Value = Self>,
    In: PassModule<K, E, Other, Fun = Fun>,
>: Sized where
    <In as ModLike>::Datum: PassDatum<K, E, Other, In>,
    Fun::Terminator: PassTerm<K, E, Other, Fun, In>,
{
    fn rewrite(&self, k: &mut Fun::FunCodeCache, b: &mut FunRef<Other>) -> Result<Val<Other>, E>;
    fn rewrite_id(
        k: &mut Fun::FunCodeCache,
        a: &Fun,
        b: &mut FunRef<Other>,
        i: ValIDFun<Fun>,
    ) -> Result<ValID<Other>, E>
    where
        ValIDFun<Fun>: Clone,
        FunId<Other>: Clone,
    {
        return k.cache(i.clone(), move |k| {
            let r = a.all()[i].rewrite(k, b)?;
            return Ok(b.fun_mut().all_mut().push(r));
        });
    }
}
pub trait PassTerm<
    K,
    E,
    Other: ModLike,
    Fun: PassFun<K, E, Other, In, Terminator = Self>,
    In: PassModule<K, E, Other, Fun = Fun>,
> where
    Fun::Value: PassVal<K, E, Other, Fun, In>,
    <In as ModLike>::Datum: PassDatum<K, E, Other, In>,
{
    fn rewrite(&self, k: &mut Fun::FunCodeCache, om: &mut FunRef<Other>) -> Result<Term<Other>, E>;
}
pub trait PassFun<K, E, Other: ModLike, In: PassModule<K, E, Other, Fun = Self>>:
    FunLike + Sized
where
    Self::Value: PassVal<K, E, Other, Self, In>,
    <In as ModLike>::Datum: PassDatum<K, E, Other, In>,
    Self::Terminator: PassTerm<K, E, Other, Self, In>,
{
    type FunCodeCache: Deref<Target = In::ModCodeCache> + DerefMut + CodeCache<In, ValID<Other>>;
    fn rewrite(&self, k: &mut In::ModCodeCache, om: &mut Other) -> FunId<Other>;
}
pub struct BasicFunCodeCache<K, I: ModLike, V> {
    re: *mut K,
    all: BTreeMap<ValID<I>, V>,
}
impl<K: Deref, I: ModLike, V> Deref for BasicFunCodeCache<K, I, V> {
    type Target = K::Target;

    fn deref(&self) -> &Self::Target {
        return unsafe { &*self.re };
    }
}
impl<K: DerefMut, I: ModLike, V> DerefMut for BasicFunCodeCache<K, I, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return unsafe { &mut *self.re };
    }
}
impl<K, I: ModLike, V: Clone> CodeCache<I, V> for BasicFunCodeCache<K, I, V>
where
    ValID<I>: Eq + Ord,
{
    fn cache<E>(
        &mut self,
        val: ValID<I>,
        go: impl FnOnce(&mut Self) -> Result<V, E>,
    ) -> Result<V, E> {
        if let Some(a) = self.all.get(&val) {
            return Ok(a.clone());
        }
        let w = go(self)?;
        self.all.insert(val, w.clone());
        return Ok(w);
    }
}
pub fn rewrite_basic_fun<
    K,
    E,
    Other: ModLike,
    Fun: PassFun<
        K,
        E,
        Other,
        In,
        Value = V,
        FunCodeCache = BasicFunCodeCache<In::ModCodeCache, In, ValID<Other>>,
    >,
    In: PassModule<K, E, Other, Fun = Fun>,
    V: PassVal<K, E, Other, Fun, In>,
>(
    s: &Fun,
    k: &mut In::ModCodeCache,
    om: &mut Other,
) -> Result<FunId<Other>, E>
where
    In::Datum: PassDatum<K, E, Other, In>,
    Fun::Terminator: PassTerm<K, E, Other, Fun, In>,
    Other::Fun: Default,
    FunId<Other>: Clone,
    ValID<Other>: Clone,
    ValID<In>: Eq + Ord,
{
    let f = om.code_mut().push(Default::default());
    PassTerm::rewrite(
        s.terminator(),
        &mut BasicFunCodeCache::<In::ModCodeCache, _, _> {
            re: k,
            all: BTreeMap::new(),
        },
        &mut FunRef {
            r#ref: om,
            fun: f.clone(),
        },
    )?;
    Ok(f)
}
pub trait PassDatum<K, E, Other: ModLike, In: PassModule<K, E, Other, Datum = Self>>
where
    <In as ModLike>::Fun: PassFun<K, E, Other, In>,
    Val<In>: PassVal<K, E, Other, <In as ModLike>::Fun, In>,
    <In::Fun as FunLike>::Terminator: PassTerm<K, E, Other, In::Fun, In>,
{
    fn rewrite(&self, k: &mut K) -> Other::Datum;
}
pub trait PassModule<K, E, Other: ModLike>: ModLike + Sized
where
    Self::Fun: PassFun<K, E, Other, Self>,
    Self::Datum: PassDatum<K, E, Other, Self>,
    <Self::Fun as FunLike>::Value: PassVal<K, E, Other, Self::Fun, Self>,
    <Self::Fun as FunLike>::Terminator: PassTerm<K, E, Other, Self::Fun, Self>,
{
    type ModCodeCache: Deref<Target = K> + DerefMut + ModCodeCache<Self, FunId<Other>, DatId<Other>>;
}
