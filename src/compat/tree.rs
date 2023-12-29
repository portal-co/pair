use std::collections::BTreeMap;

use relooper::{RelooperLabel, ShapedBlock};

use super::*;
pub struct Entry<M: ModLike> {
    pub fun: FunId<M>,
    pub args: Vec<ValID<M>>,
}
pub trait TreeTerminator<M: ModLike<Fun = F>, F: FunLike<Terminator = Self>, Err>: Sized {
    fn just(n: &mut F, x: Entry<M>) -> Result<Self, Err>;
    fn switch(n: &mut F, v: ValIDFun<F>, go: Vec<Entry<M>>, default: Entry<M>)
        -> Result<Self, Err>;
}
pub trait UnTreeTerminator<M: ModLike<Fun = F>, F: FunLike<Terminator = Self>, Err>:
    TreeTerminator<M, F, Err>
{
    fn get_tree(&self, n: &F) -> Result<Option<Tree<M, F>>, Err>;
}
pub enum Tree<M: ModLike<Fun = F>, F: FunLike> {
    Just(Entry<M>),
    Switch(ValIDFun<F>, Vec<Entry<M>>, Entry<M>),
}
pub trait Reloop<M: ModLike<Fun = Self>, Err>: FunLike + Sized
where
    Self::Terminator: UnTreeTerminator<M, Self, Err>,
    FunId<M>: RelooperLabel,
{
    fn reloop(m: &M, i: &FunId<M>) -> Result<Box<ShapedBlock<FunId<M>>>, Err> {
        let mut n = BTreeMap::new();
        Self::collect(m, i, &mut n)?;
        let mut n: Vec<_> = n.into_iter().collect();
        n.sort_by_key(|a|a.0);
        return Ok(relooper::reloop(n, *i));
    }
    fn collect(m: &M, i: &FunId<M>, go: &mut BTreeMap<FunId<M>, Vec<FunId<M>>>) -> Result<(), Err>;
}
impl<T: FunLike, M: ModLike<Fun = T>, Err> Reloop<M, Err> for T
where
    Self::Terminator: UnTreeTerminator<M, Self, Err>,
    FunId<M>: RelooperLabel,
{
    fn collect(m: &M, i: &FunId<M>, go: &mut BTreeMap<FunId<M>, Vec<FunId<M>>>) -> Result<(), Err> {
        if let Some(k) = go.get(i) {
            return Ok(());
        }
        let t = m.code()[*i].terminator().get_tree(&m.code()[*i])?;
        if let Some(t) = t {
            match t {
                Tree::Just(j) => {
                    go.insert(*i, vec![j.fun]);
                    Self::collect(m, &j.fun, go)?;
                }
                Tree::Switch(_, c, d) => {
                    let k = vec![];
                    go.insert(*i, k);
                    for j in c.into_iter().chain(vec![d]) {
                        go.get_mut(i).unwrap().push(j.fun);
                        Self::collect(m, &j.fun, go)?;
                    }
                }
            }
        }else{
            go.insert(*i, vec![]);
        }
        return Ok(());
    }
}
