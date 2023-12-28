use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

use either::Either::{self, Right, Left};
use id_arena::Id;
use relooper::{BranchMode, RelooperLabel, ShapedBlock};

// use crate::ValueDef;
use crate::utils::{my_hash, var_hash, param_hash};

use super::ValIDFun;
use super::{
    tree::{Reloop, UnTreeTerminator},
    FunId, FunLike, ModLike, Term, Val, ValID,
};

pub struct Locals {
    pub locals: BTreeSet<String>,
}
impl Locals{
    fn add(&mut self, a: String) -> String{
        self.locals.insert(a.clone());
        return a;
    }
}
pub enum Ast<In: ModLike<Fun = Fun>, Fun: FunLike<Value = Value>, Value: Statement<In>> {
    Block(Vec<Ast<In, Fun, Value>>),
    Stmt(Value::Stmt, Vec<String>, Vec<String>),
    Loop(u16, Vec<Ast<In, Fun, Value>>),
    Br(BranchMode,Option<u64>),
    Swl(BTreeMap<u64,Vec<Ast<In,Fun,Value>>>),
    Assign(String,String)
}
pub trait Statement<In: ModLike> {
    type Stmt: Clone;
    fn into_statement(&self, f: &In::Fun) -> Either<(Self::Stmt, Vec<ValID<In>>),usize>;
    fn from_statement(s: &Self::Stmt, a: &[ValID<In>], f: &mut In::Fun) -> Self;
    fn param(p: usize, f: &mut In::Fun) -> Self;
}
// impl<T: Clone,Y,R,D> Statement<crate::Module<T,Y,R,D>> for ValueDef<T,Y,R,D>{
//     type Stmt = (T,Vec<Id<D>>,Vec<Id<crate::Fun<T,Y,R,D>>>);

//     fn into_statement(&self, f: &crate::Fun<T,Y,R,D>) -> Either<(Self::Stmt, Vec<ValID<crate::Module<T,Y,R,D>>>),usize> {
//         match self{
//             ValueDef::Param(p) => Right(*p),
//             ValueDef::Emit { op, params, typ, after } => Left(((op.clone(),params.data.clone(),params.funs.clone()),params.values.iter().map(|a|match a.idx{
//                 Some(_) => todo!(),
//                 None => a.id,
//             }).collect())),
//             ValueDef::Alias(l) => f.values[l.clone().id].into_statement(f),
//         }
//     }

//     fn from_statement(s: &Self::Stmt, a: &[ValID<crate::Module<T,Y,R,D>>], f: &mut crate::Fun<T,Y,R,D>) -> Self {
//         return ValueDef::Emit { op: s.0.clone(), params: crate::Params { values: a.to_owned(), data: s.1.clone(), funs: s.2.clone() }, typ: (), after: () };
//     }

//     fn param(p: usize, f: &mut crate::Fun<T,Y,R,D>) -> Self {
//         todo!()
//     }
// }
pub trait LocalVals: FunLike{
    fn vals(&self) -> Vec<ValIDFun<Self>>;
}
pub fn all<Fun: FunLike<Value = V>,V: Statement<In>,In: ModLike<Fun = Fun>>(f: &Fun, i: ValID<In>, go: &mut BTreeSet<ValID<In>>) where ValID<In>: Clone + Eq + Ord{
    if go.contains(&i){
        return;
    }
    go.insert(i.clone());
    if let Either::Left((_,w)) = f.all()[i].into_statement(f){
        for w in w{
            all(f,w,go);
        }
    }
}
pub trait Assemble<Err>: ModLike + Sized
where
    Self::Fun: Reloop<Self, Err> + LocalVals,
    <Self::Fun as FunLike>::Terminator: UnTreeTerminator<Self, Self::Fun, Err>,
    FunId<Self>: RelooperLabel,
    ValID<Self>: Hash,
    Val<Self>: Statement<Self>,
{
    fn assemble(
        &self,
        t: &ShapedBlock<FunId<Self>>,
        l: &mut Locals,
    ) -> Result<Vec<Ast<Self, Self::Fun, Val<Self>>>, Err>;
}
impl<Err, T: ModLike> Assemble<Err> for T
where
    Self::Fun: Reloop<Self, Err> + LocalVals,
    <Self::Fun as FunLike>::Terminator: UnTreeTerminator<Self, Self::Fun, Err>,
    FunId<Self>: RelooperLabel,
    ValID<Self>: Hash + Clone,
    Val<Self>: Statement<Self>,
{
    fn assemble(
        &self,
        t: &ShapedBlock<FunId<Self>>,
        lo: &mut Locals,
    ) -> Result<Vec<Ast<Self, Self::Fun, Val<Self>>>, Err> {
        match t {
            ShapedBlock::Simple(s) => {
                let v = self.code()[s.label.clone()].vals();
                let mut w = vec![];
                for v in v{
                    let sr = self.code()[s.label.clone()].all()[v.clone()].into_statement(&self.code()[s.label.clone()]);
                    match sr{
                        Either::Left(_) => todo!(),
                        Either::Right(p) => w.push(Ast::Assign(var_hash::<Self>(s.label.clone(), v), param_hash::<Self>(s.label.clone(),p))),
                    }
                }
                match self.code()[s.label.clone()].terminator().get_tree(&self.code()[s.label.clone()]){
                    Ok(a) => match a{
                        super::tree::Tree::Just(_) => todo!(),
                        super::tree::Tree::Switch(_, _, _) => todo!(),
                    },
                    Err(_) => {},
                }
                if let Some(i) = s.immediate.as_ref(){
                    w.push(Ast::Block(self.assemble(i, lo)?))
                }
                if let Some(i) = s.next.as_ref(){
                    w.push(Ast::Block(self.assemble(i, lo)?))
                }
                 return Ok(w);
            },
            ShapedBlock::Loop(lb) => {
                let mut l = vec![Ast::Loop(lb.loop_id, self.assemble(&lb.inner,lo)?)];
                if let Some(n) = &lb.next {
                    l.append(&mut self.assemble(&n,lo)?)
                }
                return Ok(l);
            }
            ShapedBlock::Multiple(mb) => {
                let mut m = BTreeMap::new();
                for h in &mb.handled{
                    for l in &h.labels{
                        m.insert(my_hash(l), self.assemble(&h.inner,lo)?);
                    }
                }
                return Ok(vec![Ast::Swl(m)]);
            },
        }
    }
}
