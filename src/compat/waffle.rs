use either::Either::Right;
use waffle::{BlockTarget, Import, Terminator, Type, ValueDef, Operator};

use crate::utils::R;

use self::base::{BlockRef, ExportData, FuncAndBlock, GetModule, Importd, MFCache};

use super::{
    call::Call,
    tree::{Entry, TreeTerminator, UnTreeTerminator},
    typed::{TypedFunLike, TypedValue},
    FunLike, ModLike, ast::Statement,
};

pub mod base;
impl<M: GetModule> TypedValue<BlockRef<M>> for waffle::ValueDef {
    type Type = Vec<Type>;

    fn type_of(&self, f: &BlockRef<M>) -> Self::Type {
        return self.tys(&f.func().unwrap().type_pool).to_owned();
    }
}
impl<M: GetModule> TypedFunLike for BlockRef<M> {
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
impl<M: GetModule, E: Default> Call<MFCache<M>, BlockRef<MFCache<M>>, Importd, E> for ValueDef {
    fn call(
        n: &mut BlockRef<MFCache<M>>,
        f: either::Either<super::FunId<MFCache<M>>, Importd>,
        args: Vec<super::ValIDFun<BlockRef<MFCache<M>>>>,
    ) -> Result<Self, E> {
        let v = n.func_mut().r()?.arg_pool.from_iter(args.into_iter());
        let (new, sig) = match f {
            either::Either::Left(f) => {
                let f = n.cur_mut().r()?[f].to_func().r()?;
                (f, n.cur().r()?.module().funcs[f].sig())
            }
            either::Either::Right(i) => {
                let fun = n
                    .cur_mut()
                    .r()?
                    .module_mut()
                    .funcs
                    .push(waffle::FuncDecl::Import(i.sig, "$".to_owned()));
                n.cur_mut().r()?.module_mut().imports.push(Import {
                    module: i.module,
                    name: i.func,
                    kind: waffle::ImportKind::Func(fun),
                });
                (fun, i.sig)
            }
        };
        let sig = n.cur().r()?.module().signatures[sig].returns.clone();
        let t = n.func_mut().r()?.type_pool.from_iter(sig.into_iter());
        return Ok(ValueDef::Operator(
            waffle::Operator::Call {
                function_index: new,
            },
            v,
            t,
        ));
    }
}
impl<M: GetModule, E: Default> TreeTerminator<MFCache<M>, BlockRef<MFCache<M>>, E> for Terminator {
    fn just(n: &mut BlockRef<MFCache<M>>, x: super::tree::Entry<MFCache<M>>) -> Result<Self, E> {
        let f = n.k.func;
        return Ok(Terminator::Br {
            target: BlockTarget {
                args: x.args,
                block: n.cur_mut().r()?[x.fun].block_in_func(f).r()?,
            },
        });
    }

    fn switch(
        n: &mut BlockRef<MFCache<M>>,
        v: super::ValIDFun<BlockRef<MFCache<M>>>,
        mut go: Vec<super::tree::Entry<MFCache<M>>>,
        default: super::tree::Entry<MFCache<M>>,
    ) -> Result<Self, E> {
        let f = n.k.func;
        let default = BlockTarget {
            args: default.args,
            block: n.cur_mut().r()?[default.fun].block_in_func(f).r()?,
        };
        let mut params = vec![];
        for g in go.drain(..) {
            params.push(BlockTarget {
                args: g.args,
                block: n.cur_mut().r()?[g.fun].block_in_func(f).r()?,
            })
        }
        if params.len() == 1 {
            return Ok(Terminator::CondBr {
                cond: v,
                if_true: default,
                if_false: params[0].clone(),
            });
        }
        return Ok(Terminator::Select {
            value: v,
            targets: params,
            default: default,
        });
    }
}
impl<M: GetModule, E: Default> UnTreeTerminator<MFCache<M>, BlockRef<MFCache<M>>, E>
    for Terminator
{
    fn get_tree(
        &self,
        n: &BlockRef<MFCache<M>>,
    ) -> Result<super::tree::Tree<MFCache<M>, BlockRef<MFCache<M>>>, E> {
        match self {
            Terminator::Br { target } => Ok(super::tree::Tree::Just(Entry {
                fun: FuncAndBlock {
                    block: target.block,
                    func: n.k.func,
                },
                args: target.args.clone(),
            })),
            Terminator::CondBr {
                cond,
                if_true,
                if_false,
            } => Ok(super::tree::Tree::Switch(
                cond.clone(),
                vec![Entry {
                    fun: FuncAndBlock {
                        block: if_false.block,
                        func: n.k.func,
                    },
                    args: if_false.args.clone(),
                }],
                Entry {
                    fun: FuncAndBlock {
                        block: if_true.block,
                        func: n.k.func,
                    },
                    args: if_true.args.clone(),
                },
            )),
            Terminator::Select {
                value,
                targets,
                default,
            } => {
                let mut t2 = vec![];
                for target in targets {
                    t2.push(Entry {
                        fun: FuncAndBlock {
                            block: target.block,
                            func: n.k.func,
                        },
                        args: target.args.clone(),
                    });
                }
                return Ok(super::tree::Tree::Switch(
                    value.clone(),
                    t2,
                    Entry {
                        fun: FuncAndBlock {
                            block: default.block,
                            func: n.k.func,
                        },
                        args: default.args.clone(),
                    },
                ));
            }
            Terminator::Return { values } => Err(Default::default()),
            Terminator::Unreachable => Err(Default::default()),
            Terminator::None => Err(Default::default()),
        }
    }
}
impl<M: GetModule> Statement<MFCache<M>> for ValueDef{
    type Stmt = Operator;

    fn into_statement(&self, f: &<MFCache<M> as ModLike>::Fun) -> either::Either<(Self::Stmt, Vec<super::ValID<MFCache<M>>>),usize> {
        todo!()
    }

    fn from_statement(s: &Self::Stmt, a: &[super::ValID<MFCache<M>>], f: &mut <MFCache<M> as ModLike>::Fun) -> Self {
        todo!()
    }

    fn param(p: usize, f: &mut <MFCache<M> as ModLike>::Fun) -> Self {
        todo!()
    }

}