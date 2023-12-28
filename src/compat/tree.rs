use super::*;
pub struct Entry<M: ModLike>{
    pub fun: FunId<M>,
    pub args: Vec<ValID<M>>
}
pub trait TreeTerminator<M: ModLike<Fun = F>,F: FunLike<Terminator = Self>,Err>: Sized{
    fn just(n: &mut F, x: Entry<M>) -> Result<Self,Err>;
    fn switch(n: &mut F, v: ValIDFun<F>, go: Vec<Entry<M>>, default: Entry<M>) -> Result<Self,Err>;
}