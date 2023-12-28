use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::hash::*;

use relooper::RelooperLabel;

use super::{
    tree::{Reloop, UnTreeTerminator},
    FunId, ModLike, ValID, ValIDFun,
};
fn my_hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}
pub fn var_hash<In: ModLike>(f: FunId<In>, v: ValID<In>) -> String
where
    FunId<In>: Hash,
    ValID<In>: Hash,
{
    return format!("${}", my_hash((f, v)));
}

pub trait REV<In: ModLike<Fun = Fun>, Fun: RustEmit<In, Err, Value = Self>, Err>: Sized
where
    Fun::Terminator: UnTreeTerminator<In, Fun, Err>,
    FunId<In>: RelooperLabel,
    ValID<In>: Hash,
{
    fn emit(r#mod: &In, fun: FunId<In>, val: ValIDFun<Fun>, maps: BTreeMap<ValIDFun<Fun>,String>) -> syn::Expr;
    
}
pub trait RustEmit<In: ModLike<Fun = Self>, Err>: Reloop<In, Err>
where
    Self::Terminator: UnTreeTerminator<In, Self, Err>,
    FunId<In>: RelooperLabel,
    ValID<In>: Hash,
    Self::Value: REV<In, Self, Err>,
{
}
