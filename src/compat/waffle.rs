use waffle::Type;

use self::base::{BlockRef, ExportData, GetModule, MFCache};

use super::{FunLike, ModLike, typed::{TypedValue, TypedFunLike}};

pub mod base;
impl<M: GetModule> TypedValue<BlockRef<M>> for waffle::ValueDef{
    type Type = Vec<Type>;

    fn type_of(&self, f: &BlockRef<M>) -> Self::Type {
        return self.tys(&f.func().unwrap().type_pool).to_owned();
    }
}
impl<M: GetModule> TypedFunLike for BlockRef<M>{
    type Type = Vec<Type>;
}
impl<M: GetModule> FunLike for BlockRef<M> {
    type Value = waffle::ValueDef;

    type Arena = Self;

    fn all(&self) -> &Self::Arena {
        self
    }

    fn all_mut(&mut self) -> &mut Self::Arena {
        self
    }

    type Terminator = waffle::Terminator;

    fn terminator(&self) -> &Self::Terminator {
        return &self.func().unwrap().blocks[self.k.block].terminator;
    }

    fn terminator_mut(&mut self) -> &mut Self::Terminator {
        let k = self.k.block;
        return &mut self.func_mut().unwrap().blocks[k].terminator;
    }
}
impl<M: GetModule> ModLike for MFCache<M> {
    type Fun = BlockRef<MFCache<M>>;

    type Code = Self;

    fn code(&self) -> &Self::Code {
        self
    }

    fn code_mut(&mut self) -> &mut Self::Code {
        self
    }

    type Datum = ExportData;

    type Data = Self;

    fn data(&self) -> &Self::Data {
        self
    }

    fn data_mut(&mut self) -> &mut Self::Data {
        self
    }
}
