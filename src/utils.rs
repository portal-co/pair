use either::Either;

use crate::{compat::{FunLike, ArenaLike, ValIDFun, ModLike, FunId}, Value, ValueDef};

#[cfg(feature = "waffle")]
pub mod waffle;
#[cfg(feature = "rust")]
pub mod rust;

pub enum FunCommon<X,F: FunLike,K>{
    Extern(X),
    Fun(K),
    Bind(Box<FunCommon<X,F,K>>, Vec<ValIDFun<F>>)
}


pub trait R<Err>{
    type Ty;
    fn r(self) -> Result<Self::Ty,Err>;
}
impl<Err: Default,T> R<Err> for Option<T>{
    type Ty = T;

    fn r(self) -> Result<Self::Ty,Err> {
        return match self{
            Some(a) =>Ok(a),
            None => Err(Default::default()),
        };
    }
} 