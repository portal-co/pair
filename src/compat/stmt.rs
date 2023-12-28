use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

use either::Either::{self, Left, Right};
use id_arena::Id;
use relooper::{BranchMode, RelooperLabel, ShapedBlock};

// use crate::ValueDef;
use crate::utils::{my_hash, param_hash, var_hash};

use super::ValIDFun;
use super::{
    tree::{Reloop, UnTreeTerminator},
    FunId, FunLike, ModLike, Term, Val, ValID,
};

pub enum Stmt<S: Statement<In>, In: ModLike> {
    Basic(S::Stmt, Vec<ValID<In>>),
    Param(usize),
    Pick(ValID<In>, usize),
}
pub trait Statement<In: ModLike>: Sized {
    type Stmt: Clone;
    fn into_statement(&self, f: &In::Fun) -> Stmt<Self, In>;
    fn from_statement(s: &Stmt<Self, In>, f: &mut In::Fun) -> Self;
}
