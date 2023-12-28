use std::ops::{Index, IndexMut};

use id_arena::{Arena, Id};

use crate::{Fun, Module, ValueDef};
pub mod call;
#[cfg(feature = "rust")]
pub mod rust;
pub mod typed;
#[cfg(feature = "waffle")]
pub mod waffle;
pub unsafe fn unbound<'a, 'b, T>(a: &'a mut T) -> &'b mut T {
    std::mem::transmute(a)
}

pub trait ArenaLike<T>: Index<Self::Id, Output = T> + IndexMut<Self::Id, Output = T> {
    type Id;
    fn push(&mut self, a: T) -> Self::Id;
}
impl<T> ArenaLike<T> for Arena<T> {
    type Id = Id<T>;

    fn push(&mut self, a: T) -> Self::Id {
        return self.alloc(a);
    }
}
pub trait FunLike {
    type Value;
    type Arena: ArenaLike<Self::Value>;
    fn all(&self) -> &Self::Arena;
    fn all_mut(&mut self) -> &mut Self::Arena;
    type Terminator;
    fn terminator(&self) -> &Self::Terminator;
    fn terminator_mut(&mut self) -> &mut Self::Terminator;
}
impl<T, Y, R, D> FunLike for Fun<T, Y, R, D> {
    type Value = ValueDef<T, Y, R, D>;

    type Arena = Arena<Self::Value>;

    fn all(&self) -> &Self::Arena {
        return &self.values;
    }

    fn all_mut(&mut self) -> &mut Self::Arena {
        return &mut self.values;
    }

    type Terminator = R;

    fn terminator(&self) -> &Self::Terminator {
        return &self.terminator;
    }

    fn terminator_mut(&mut self) -> &mut Self::Terminator {
        return &mut self.terminator;
    }
}
pub trait ModLike {
    type Fun: FunLike;
    type Code: ArenaLike<Self::Fun>;
    fn code(&self) -> &Self::Code;
    fn code_mut(&mut self) -> &mut Self::Code;
    type Datum;
    type Data: ArenaLike<Self::Datum>;
    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self) -> &mut Self::Data;
}
impl<T, Y, R, D> ModLike for Module<T, Y, R, D> {
    type Fun = Fun<T, Y, R, D>;

    type Code = Arena<Fun<T, Y, R, D>>;

    fn code(&self) -> &Self::Code {
        return &self.code;
    }

    fn code_mut(&mut self) -> &mut Self::Code {
        return &mut self.code;
    }

    type Datum = D;

    type Data = Arena<D>;

    fn data(&self) -> &Self::Data {
        return &self.data;
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        return &mut self.data;
    }
}
pub type ValIDFun<F: FunLike> = <F::Arena as ArenaLike<F::Value>>::Id;
pub type ValID<A: ModLike> = ValIDFun<A::Fun>;
pub type FunId<A: ModLike> = <A::Code as ArenaLike<A::Fun>>::Id;
pub type DatId<A: ModLike> = <A::Data as ArenaLike<A::Datum>>::Id;
