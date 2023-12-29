use waffle::Module;

use crate::compat::{FunLike, ModLikeIter};
use crate::{
    compat::{
        tree::{Reloop, UnTreeTerminator},
        waffle::base::{BlockRef, MFCache},
        ModLike,
    },
    utils::waffle::parse,
};
fn mod1() -> Module<'static> {
    return parse(include_bytes!("./mod1.wasm")).unwrap();
}
fn test_reloop(m: Module<'static>) {
    let m = MFCache::from_inner(m);
    for n in m.keys() {
        let t: Result<_,()> = m[n].terminator().get_tree(&m[n]);
        assert!(t.is_ok(), "tree deconstruction should succeed");
        if let Some(_) = t.unwrap() {
            let r: Result<_, ()> = BlockRef::<MFCache<Module<'static>>>::reloop(&*m, &n);
            assert!(r.is_ok(), "relooping should succeed");
        }
    }
}
#[test]
fn mod1_reloop() {
    test_reloop(mod1());
}
