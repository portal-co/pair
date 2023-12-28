use either::Either;

use super::*;

pub trait Call<M: ModLike<Fun = F>, F: FunLike<Value = Self>, X, Err>: Sized {
    fn call(n: &mut F, f: Either<FunId<M>, X>, args: Vec<ValIDFun<F>>) -> Result<Self, Err>;
}
pub trait CallIndirect<M: ModLike<Fun = F>, F: FunLike<Value = Self>, X, Err>:
    Call<M, F, X, Err>
{
    fn fun(n: &mut F, f: FunId<M>) -> Result<Self, Err>;
    fn r#extern(m: &mut F, x: X) -> Result<Self, Err>;
    fn call_indirect(n: &mut F, v: ValIDFun<F>, r: Vec<ValIDFun<F>>) -> Result<Self, Err>;
    fn call(n: &mut F, f: Either<FunId<M>, X>, args: Vec<ValIDFun<F>>) -> Result<Self, Err> {
        let a = match f {
            Either::Left(f) => Self::fun(n, f)?,
            Either::Right(x) => Self::r#extern(n, x)?,
        };
        let a = n.all_mut().push(a);
        return Self::call_indirect(n, a, args);
    }
}
pub trait BindFun<M: ModLike<Fun = F>, F: FunLike<Value = Self>, X, E>:
    CallIndirect<M, F, X, E>
{
    fn bind(n: &mut F, v: ValIDFun<F>, all: Vec<ValIDFun<F>>) -> Result<Self, E>;
    fn call_one(n: &mut F, v: ValIDFun<F>) -> Result<Self, E>;
    fn call_indirect(n: &mut F, v: ValIDFun<F>, r: Vec<ValIDFun<F>>) -> Result<Self, E> {
        let a = Self::bind(n, v, r)?;
        let a = n.all_mut().push(a);
        return Self::call_one(n, a);
    }
}
