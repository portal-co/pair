use waffle::{Type, ValueDef, Import};

use crate::utils::R;

use self::base::{BlockRef, ExportData, GetModule, MFCache, Importd};

use super::{FunLike, ModLike, typed::{TypedValue, TypedFunLike}, call::Call};

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
impl<M: GetModule,E: Default> Call<MFCache<M>,BlockRef<MFCache<M>>,Importd,E> for ValueDef{
    fn call(n: &mut BlockRef<MFCache<M>>, f: either::Either<super::FunId<MFCache<M>>,Importd>, args: Vec<super::ValIDFun<BlockRef<MFCache<M>>>>) -> Result<Self,E> {
        let v = n.func_mut().r()?.arg_pool.from_iter(args.into_iter());
        let (new,sig) = match f{
            either::Either::Left(f) => {
                let f = n.cur_mut().r()?[f].to_func().r()?;
                (f,n.cur().r()?.module().funcs[f].sig())
            },
            either::Either::Right(i) => {
                let fun = n.cur_mut().r()?.module_mut().funcs.push(waffle::FuncDecl::Import(i.sig, "$".to_owned()));
                n.cur_mut().r()?.module_mut().imports.push(Import{
                    module: i.module,
                    name: i.func,
                    kind: waffle::ImportKind::Func(fun)
                });
                (fun,i.sig)
            },
        };
        let sig = n.cur().r()?.module().signatures[sig].returns.clone();
        let t = n.func_mut().r()?.type_pool.from_iter(sig.into_iter());
        return Ok(ValueDef::Operator(waffle::Operator::Call { function_index: new }, v, t));
    }
}