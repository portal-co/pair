use std::{
    cell::UnsafeCell,
    collections::BTreeMap,
    ops::{Deref, DerefMut, Index, IndexMut}, marker::PhantomPinned, pin::Pin,
};

use waffle::{
    Block, BlockTarget, Export, ExportKind, Func, FuncDecl, FunctionBody, Global, GlobalData,
    Memory, MemoryData, Signature, SignatureData, Table, TableData, Type, Value, ValueDef,
};

use crate::{compat::{ArenaLike, OrderedArenaLike}, utils::waffle::clone_fn};
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Default, Debug, Hash)]
pub struct FuncAndBlock {
    pub func: waffle::Func,
    pub block: waffle::Block,
}

pub struct FuncRef<M> {
    cur: *mut M,
    pub func: Func,
}
impl<M> FuncRef<M> {
    pub fn r#in(self, b: Block) -> BlockRef<M> {
        return BlockRef {
            cur: self.cur,
            k: FuncAndBlock {
                func: self.func,
                block: b,
            },
        };
    }
}
pub struct BlockRef<M> {
    cur: *mut M,
    pub k: FuncAndBlock,
}
impl<M> Default for BlockRef<M> {
    fn default() -> Self {
        Self {
            cur: std::ptr::null_mut(),
            k: Default::default(),
        }
    }
}
impl<M> BlockRef<M> {
    pub fn into_func_ref(self) -> (FuncRef<M>, Block) {
        return (
            FuncRef {
                cur: self.cur,
                func: self.k.func,
            },
            self.k.block,
        );
    }
}
pub trait GetModule {
    fn module(&self) -> &waffle::Module<'static>;
    fn module_mut(&mut self) -> &mut waffle::Module<'static>;
}
impl GetModule for waffle::Module<'static>{
    fn module(&self) -> &waffle::Module<'static> {
        self
    }

    fn module_mut(&mut self) -> &mut waffle::Module<'static> {
        self
    }
}
impl<M: GetModule> Index<waffle::Value> for BlockRef<M> {
    type Output = waffle::ValueDef;

    fn index(&self, index: waffle::Value) -> &Self::Output {
        return &self.func().unwrap().values[index];
    }
}
impl<M: GetModule> IndexMut<waffle::Value> for BlockRef<M> {
    fn index_mut(&mut self, index: waffle::Value) -> &mut Self::Output {
        return &mut self.func_mut().unwrap().values[index];
    }
}
impl<M: GetModule> ArenaLike<waffle::ValueDef> for BlockRef<M> {
    type Id = waffle::Value;

    fn push(&mut self, a: waffle::ValueDef) -> Self::Id {
        return self.add(a).unwrap();
    }
}
impl<M: GetModule> OrderedArenaLike<waffle::ValueDef> for BlockRef<M>{
    fn push_after(&mut self, a: waffle::ValueDef, after: Self::Id) -> Self::Id {
        return self.add_after(a, Some(after)).unwrap();
    }

    fn push_just_before(&mut self, a: waffle::ValueDef, before: Self::Id) -> Self::Id {
        return self.add_just_before(a, before).unwrap();
    }
}
impl<M: GetModule> BlockRef<M> {
    pub fn cur(&self) -> Option<&M> {
        if self.cur.is_null() {
            return None;
        }
        return unsafe { self.cur.as_ref() };
    }
    pub fn cur_mut(&mut self) -> Option<&mut M> {
        if self.cur.is_null() {
            return None;
        }
        return unsafe { self.cur.as_mut() };
    }
    pub fn func(&self) -> Option<&FunctionBody> {
        return self.cur()?.module().funcs[self.k.func].body();
    }
    pub fn func_mut(&mut self) -> Option<&mut FunctionBody> {
        let f = self.k.func;
        return self.cur_mut()?.module_mut().funcs[f].body_mut();
    }
    pub fn add(&mut self, a: ValueDef) -> Option<Value>{
        return self.add_after(a, None);
    }
    pub fn add_after(&mut self, a: ValueDef, after: Option<Value>) -> Option<Value>{
        let v = self.func_mut()?.add_value(a);
        let l = self.k.block;
        let mut i = std::mem::take(&mut self.func_mut()?.blocks[l].insts);
        for j in i{
            self.func_mut()?.blocks[l].insts.push(j);
            if let Some(after) = after{
            if j == after{
                self.func_mut()?.append_to_block(l, v);
            }
        }
        }
        if let None = after{
            self.func_mut()?.append_to_block(l, v);
        }
        return Some(v);
    }
    pub fn add_just_before(&mut self, a: ValueDef, before: Value) -> Option<Value>{
        let v = self.func_mut()?.add_value(a);
        let l = self.k.block;
        let mut i = std::mem::take(&mut self.func_mut()?.blocks[l].insts);
        for j in i{
            if j == before{
                self.func_mut()?.append_to_block(l, v);
            }
            self.func_mut()?.blocks[l].insts.push(j);
        }
        return Some(v);
    }
    pub fn params(&self) -> Option<Vec<(Type, Value)>> {
        let l = self.k.block;
        return Some(self.func()?.blocks[self.k.block].params.clone());
    }
    pub fn add_param(&mut self, t: Type) -> Option<Value> {
        let l = self.k.block;
        return Some(self.func_mut()?.add_blockparam(l, t));
    }
    pub fn in_func(self, target: Func) -> Option<BlockRef<M>> {
        if self.k.func == target {
            return Some(self);
        }
        if self.cur.is_null() {
            return None;
        }
        let r = unsafe { (&mut *self.cur).module_mut().funcs[target].body_mut()? };
        // let s = unsafe { (&*self.cur).funcs[self.k.func].body()? };
        let lr = clone_fn(r, self.func()?);
        return Some(BlockRef {
            cur: self.cur,
            k: FuncAndBlock {
                func: target,
                block: *lr.all.get(&self.k.block)?,
            },
        });
    }
    pub fn block_in_func(&mut self, target: Func) -> Option<Block> {
        if self.k.func == target {
            return Some(self.k.block);
        }
        if self.cur.is_null() {
            return None;
        }
        let r = unsafe { (&mut *self.cur).module_mut().funcs[target].body_mut()? };
        // let s = unsafe { (&*self.cur).funcs[self.k.func].body()? };
        let lr = clone_fn(r, self.func()?);
        return Some(*lr.all.get(&self.k.block)?);
    }
    pub fn to_func(&mut self) -> Option<Func> {
        let s = SignatureData {
            returns: self.func()?.rets.clone(),
            params: self.func()?.blocks[self.k.block]
                .params
                .iter()
                .map(|a| a.0)
                .collect(),
        };
        let mut n = self.func()?.clone();
        n.entry = self.k.block;
        let s = unsafe { &mut *self.cur }.module_mut().signatures.push(s);
        return Some(
            unsafe { &mut *self.cur }
                .module_mut()
                .funcs
                .push(waffle::FuncDecl::Body(s, "$".to_owned(), n)),
        );
    }
}
#[derive(Clone, Debug)]
pub enum ExportData {
    Table(TableData),
    Global(GlobalData),
    Memory(MemoryData),
}
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ExportKey {
    Table(Table),
    Global(Global),
    Memory(Memory),
}

pub struct MFCache<M: GetModule> {
    ptr: Option<M>,
    cache: UnsafeCell<BTreeMap<FuncAndBlock, BlockRef<MFCache<M>>>>,
    data_cache: UnsafeCell<BTreeMap<ExportKey, ExportData>>,
    _pinned: PhantomPinned,
}
impl<M: GetModule> Drop for MFCache<M> {
    fn drop(&mut self) {
        if let None = self.ptr {
            return;
        }
        self.flush();
    }
}
impl<M: GetModule> MFCache<M> {
    pub fn create_export(&self, index: ExportKey) -> ExportData {
        match index {
            ExportKey::Table(t) => ExportData::Table(self.module().tables[t].clone()),
            ExportKey::Global(g) => ExportData::Global(self.module().globals[g].clone()),
            ExportKey::Memory(m) => ExportData::Memory(self.module().memories[m].clone()),
        }
    }
    pub fn into_inner(mut self) -> M {
        self.flush();
        return self.ptr.take().unwrap();
    }
    pub fn from_inner(m: M) -> Pin<Box<Self>> {
        return Box::pin(Self {
            ptr: Some(m),
            cache: UnsafeCell::new(BTreeMap::new()),
            data_cache: UnsafeCell::new(BTreeMap::new()),
            _pinned: PhantomPinned,
        });
    }
    pub fn flush(&mut self) {
        for (k, v) in std::mem::take(self.data_cache.get_mut()) {
            match (k, v) {
                (ExportKey::Table(t), ExportData::Table(d)) => self.module_mut().tables[t] = d,
                (ExportKey::Global(g), ExportData::Global(d)) => self.module_mut().globals[g] = d,
                (ExportKey::Memory(m), ExportData::Memory(d)) => self.module_mut().memories[m] = d,
                _ => panic!("invalid key state"),
            }
        }
    }
    pub fn alloc_block(&mut self, sig: SignatureData) -> FuncAndBlock {
        let s = self.module_mut().signatures.push(sig);
        let f = FunctionBody::new(self.module(), s);
        let e = f.entry.clone();
        return FuncAndBlock {
            block: e,
            func: self
                .module_mut()
                .funcs
                .push(waffle::FuncDecl::Body(s, "$".to_owned(), f)),
        };
    }
}
impl<M: GetModule> GetModule for MFCache<M> {
    fn module(&self) -> &waffle::Module<'static> {
        return self.ptr.as_ref().unwrap().module();
    }

    fn module_mut(&mut self) -> &mut waffle::Module<'static> {
        return self.ptr.as_mut().unwrap().module_mut();
    }
}
impl<M: GetModule> Index<FuncAndBlock> for MFCache<M> {
    type Output = BlockRef<MFCache<M>>;

    fn index(&self, index: FuncAndBlock) -> &Self::Output {
        return unsafe { &mut *self.cache.get() }
            .entry(index)
            .or_insert_with(|| BlockRef {
                cur: self as *const MFCache<M> as *mut MFCache<M>,
                k: index,
            });
    }
}
impl<M: GetModule> IndexMut<FuncAndBlock> for MFCache<M> {
    fn index_mut(&mut self, index: FuncAndBlock) -> &mut Self::Output {
        return unsafe { &mut *self.cache.get() }
            .entry(index)
            .or_insert_with(|| BlockRef {
                cur: self as *const MFCache<M> as *mut MFCache<M>,
                k: index,
            });
    }
}
impl<M: GetModule> ArenaLike<BlockRef<MFCache<M>>> for MFCache<M> {
    type Id = FuncAndBlock;

    fn push(&mut self, a: BlockRef<MFCache<M>>) -> Self::Id {
        if a.cur.is_null() {
            return self.alloc_block(SignatureData {
                params: vec![],
                returns: vec![],
            });
        };
        return a.k;
    }
}
impl<M: GetModule> Index<ExportKey> for MFCache<M> {
    type Output = ExportData;

    fn index(&self, index: ExportKey) -> &Self::Output {
        return unsafe { &mut *self.data_cache.get() }
            .entry(index.clone())
            .or_insert_with(|| self.create_export(index));
    }
}
impl<M: GetModule> IndexMut<ExportKey> for MFCache<M> {
    fn index_mut(&mut self, index: ExportKey) -> &mut Self::Output {
        return unsafe { &mut *self.data_cache.get() }
            .entry(index.clone())
            .or_insert_with(|| self.create_export(index));
    }
}
impl<M: GetModule> ArenaLike<ExportData> for MFCache<M> {
    type Id = ExportKey;

    fn push(&mut self, a: ExportData) -> Self::Id {
        match a {
            ExportData::Table(t) => ExportKey::Table(self.module_mut().tables.push(t)),
            ExportData::Global(g) => ExportKey::Global(self.module_mut().globals.push(g)),
            ExportData::Memory(m) => ExportKey::Memory(self.module_mut().memories.push(m)),
        }
    }
}
pub struct Importd {
    pub module: String,
    pub func: String,
    pub sig: Signature,
}
