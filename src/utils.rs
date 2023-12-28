use either::Either;

use crate::{
    compat::{ArenaLike, FunId, FunLike, ModLike, ValIDFun, ValID},
};
use std::{hash::*, collections::hash_map::DefaultHasher};

#[cfg(feature = "rust")]
pub mod rust;
#[cfg(feature = "waffle")]
pub mod waffle;

pub enum FunCommon<X, F: FunLike, K> {
    Extern(X),
    Fun(K),
    Bind(Box<FunCommon<X, F, K>>, Vec<ValIDFun<F>>),
}

pub trait R<Err> {
    type Ty;
    fn r(self) -> Result<Self::Ty, Err>;
}
impl<Err: Default, T> R<Err> for Option<T> {
    type Ty = T;

    fn r(self) -> Result<Self::Ty, Err> {
        return match self {
            Some(a) => Ok(a),
            None => Err(Default::default()),
        };
    }
}
pub fn my_hash<T>(obj: T) -> u64
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
    return format!("v${}", my_hash((f, v)));
}
pub fn param_hash<In: ModLike>(f: FunId<In>, v: usize) -> String
where
    FunId<In>: Hash,
{
    return format!("p${}", my_hash((f, v)));
}