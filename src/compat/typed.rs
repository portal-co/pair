use crate::{Fun, ValueDef};

use super::FunLike;
pub trait TypedValue<F: TypedFunLike<Value = Self, Type = Self::Type>>: Sized {
    type Type;
    fn type_of(&self, f: &F) -> Self::Type;
}
pub trait TypedFunLike: FunLike + Sized
where
    Self::Value: TypedValue<Self, Type = Self::Type> + Sized,
{
    type Type;
}
pub trait Slice: Sized {
    fn bind(a: Vec<Self>) -> Self;
    fn slice(self) -> Result<Vec<Self>, Self>;
}
impl<Y> Slice for Vec<Y> {
    fn bind(a: Vec<Self>) -> Self {
        return a.into_iter().flatten().collect();
    }

    fn slice(self) -> Result<Vec<Self>, Self> {
        return Ok(self.into_iter().map(|a| vec![a]).collect());
    }
}
impl<T, Y: Clone + Slice, R, D> TypedValue<Fun<T, Y, R, D>> for ValueDef<T, Y, R, D> {
    type Type = Y;

    fn type_of(&self, f: &Fun<T, Y, R, D>) -> Self::Type {
        match self {
            ValueDef::Param(p) => f.input_ty[*p].clone(),
            ValueDef::Emit {
                op,
                params,
                typ,
                after,
            } => typ.clone(),
            ValueDef::Alias(l) => {
                let mut a = f.values[l.id].type_of(f);
                if let Some(i) = l.idx {
                    a = a.slice().map_err(|_| "not multi value").unwrap()[i as usize].clone();
                }
                a
            }
        }
    }
}
impl<T, Y: Clone + Slice, R, D> TypedFunLike for Fun<T, Y, R, D> {
    type Type = Y;
}
