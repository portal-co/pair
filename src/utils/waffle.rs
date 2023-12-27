use std::collections::BTreeMap;

use waffle::{
    cfg::CFGInfo, pool::ListRef, Block, BlockTarget, Func, FunctionBody, Memory, MemoryArg, Module,
    Operator, Signature, SignatureData, Terminator, Type, Value, ValueDef,
};

pub fn tweak_value(
    f: &mut FunctionBody,
    x: &mut ValueDef,
    mut m: impl FnMut(&mut Value),
    b: Block,
) {
    match x {
        ValueDef::BlockParam(a, _, _) => *a = b,
        ValueDef::Operator(_, l, _) => {
            *l = f.arg_pool.deep_clone(l.clone());
            for v in &mut f.arg_pool[l.clone()] {
                m(v)
            }
        }
        ValueDef::PickOutput(v, _, _) => m(v),
        ValueDef::Alias(v) => m(v),
        ValueDef::Placeholder(_) => todo!(),
        ValueDef::Trace(_, l) => {
            *l = f.arg_pool.deep_clone(l.clone());
            for v in &mut f.arg_pool[l.clone()] {
                m(v)
            }
        }
        ValueDef::None => todo!(),
    }
}
pub fn tweak_target(
    f: &mut FunctionBody,
    x: &mut BlockTarget,
    mut m: impl FnMut(&mut Value),
    mut k: impl FnMut(&mut Block),
) {
    k(&mut x.block);
    for a in &mut x.args {
        m(a)
    }
}
pub fn tweak_terminator(
    f: &mut FunctionBody,
    x: &mut Terminator,
    mut m: impl FnMut(&mut Value),
    mut k: impl FnMut(&mut Block),
) {
    match x {
        Terminator::Br { target } => tweak_target(f, target, m, k),
        Terminator::CondBr {
            cond,
            if_true,
            if_false,
        } => {
            m(cond);
            tweak_target(f, if_true, &mut m, &mut k);
            tweak_target(f, if_false, m, k);
        }
        Terminator::Select {
            value,
            targets,
            default,
        } => {
            m(value);
            for target in targets {
                tweak_target(f, target, &mut m, &mut k)
            }
            tweak_target(f, default, m, k)
        }
        Terminator::Return { values } => {
            for a in values {
                m(a)
            }
        }
        Terminator::Unreachable => {}
        Terminator::None => {}
    }
}
pub fn clone_value(
    basis: &FunctionBody,
    f: &mut FunctionBody,
    mut m: impl FnMut(&mut Value),
    v: Value,
    b: Block,
) -> Value {
    let mut w = basis.values.get(v).unwrap().clone();
    tweak_value(f, &mut w, m, b);
    return f.add_value(w);
}
pub fn clone_block(
    f: &mut FunctionBody,
    basis: &FunctionBody,
    b: Block,
    new: Block,
    mut k: impl FnMut(&mut Block),
) {
    let mut d = basis.blocks.get(b).unwrap().clone();
    let mut m: BTreeMap<Value, Value> = BTreeMap::new();
    let r = new;
    for (pt, pv) in d.params.clone() {
        m.insert(pv, f.add_blockparam(r, pt));
    }
    for v in &mut d.insts {
        let n = clone_value(
            basis,
            f,
            |a| {
                *a = match m.get(&a) {
                    None => a.clone(),
                    Some(b) => b.clone(),
                }
            },
            v.clone(),
            r,
        );
        m.insert((*v).clone(), n.clone());
        *v = n;
        f.append_to_block(r, n);
    }
    tweak_terminator(
        f,
        &mut d.terminator,
        |a| {
            *a = match m.get(&a) {
                None => a.clone(),
                Some(b) => b.clone(),
            }
        },
        k,
    );
    // let mut c = f.blocks.get_mut(r).unwrap();
    // for a in d.insts.clone(){
    //     f.append_to_block(r, a);
    // }
    f.set_terminator(r, d.terminator.clone());
}
pub struct FunCloneRes {
    pub all: BTreeMap<Block, Block>,
}
pub fn clone_fn(f: &mut FunctionBody, basis: &FunctionBody) -> FunCloneRes {
    let mut all = BTreeMap::new();
    for k in basis.blocks.entries().map(|a| a.0) {
        all.insert(k, f.add_block());
    }
    for (a, b) in all.clone() {
        clone_block(f, basis, a, b, |k| *k = *all.get(k).unwrap());
    }
    return FunCloneRes { all };
}
