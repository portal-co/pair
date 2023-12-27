use self::base::{BlockRef, GetModule, MFCache, ExportData};

use super::{FunLike, ModLike};

pub mod base;
impl<M: GetModule> FunLike for BlockRef<M>{
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
impl<M: GetModule> ModLike for MFCache<M>{
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