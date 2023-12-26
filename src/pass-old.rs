
pub struct ModuleTransform<T, Y, R, D, S, L, E, P> {
    pub input: Module<T, Y, R, D>,
    pub out: Module<S, L, E, P>,
    pub datum_cache: BTreeMap<Id<D>, Id<P>>,
    pub code_cache: BTreeMap<Id<Fun<T, Y, R, D>>, Id<Fun<S, L, E, P>>>,
}
pub struct FuncTransformCtx<T, Y, R, D, S, L, E, P> {
    pub input: Id<Fun<T, Y, R, D>>,
    pub output: Id<Fun<S, L, E, P>>,
}
pub trait MTBehavior<T, Y, R, D, S, L, E, P> {
    fn value(
        &mut self,
        ctx: &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
        fun_ctx: FuncTransformCtx<T, Y, R, D, S, L, E, P>,
        def: ValueDef<T, Y, R, D>,
        value: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<ValueDef<T, Y, R, D>>,
        ) -> Id<ValueDef<S, L, E, P>>,
        func: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<Fun<T, Y, R, D>>,
        ) -> Id<Fun<S, L, E, P>>,
        dat: impl FnMut(&mut Self, &mut ModuleTransform<T, Y, R, D, S, L, E, P>, Id<D>) -> Id<P>,
    ) -> ValueDef<S, L, E, P>;
    fn terminator(
        &mut self,
        ctx: &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
        fun_ctx: FuncTransformCtx<T, Y, R, D, S, L, E, P>,
        def: R,
        value: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<ValueDef<T, Y, R, D>>,
        ) -> Id<ValueDef<S, L, E, P>>,
        func: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<Fun<T, Y, R, D>>,
        ) -> Id<Fun<S, L, E, P>>,
        dat: impl FnMut(&mut Self, &mut ModuleTransform<T, Y, R, D, S, L, E, P>, Id<D>) -> Id<P>,
    ) -> E;
    fn datum(
        &mut self,
        ctx: &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
        def: D,
        func: impl FnMut(
            &mut Self,
            &mut ModuleTransform<T, Y, R, D, S, L, E, P>,
            Id<Fun<T, Y, R, D>>,
        ) -> Id<Fun<S, L, E, P>>,
        dat: impl FnMut(&mut Self, &mut ModuleTransform<T, Y, R, D, S, L, E, P>, Id<D>) -> Id<P>,
    ) -> P;
}
pub type ValueTransMap<T, Y, R, D, S, L, E, P> =
    Rc<RefCell<BTreeMap<Id<ValueDef<T, Y, R, D>>, Id<ValueDef<S, L, E, P>>>>>;
impl<T: Clone, Y: Clone, R: Clone, D: Clone, S, L, E: Default, P>
    ModuleTransform<T, Y, R, D, S, L, E, P>
{
    pub fn func_value<W: MTBehavior<T, Y, R, D, S, L, E, P>>(
        &mut self,
        w: &mut W,
        f: Id<Fun<T, Y, R, D>>,
        m: ValueTransMap<T, Y, R, D, S, L, E, P>,
        a: Id<ValueDef<T, Y, R, D>>,
    ) -> Id<ValueDef<S, L, E, P>> {
        {
            if let Some(x) = m.borrow_mut().get(&a) {
                return *x;
            }
        }
        let i = self.out.code[*self.code_cache.get(&f).unwrap()]
            .values
            .alloc(ValueDef::Param(usize::MAX));
        let v = {
            {
                m.borrow_mut().insert(a.clone(), i.clone())
            };
            w.value(
                self,
                FuncTransformCtx {
                    input: f.clone(),
                    output: *self.code_cache.get(&f).unwrap(),
                },
                self.input.code[f].values[a].clone(),
                |w, t, v| t.func_value(w, f, m.clone(), v),
                |w, t, f| t.func(w, f),
                |w, t, d| t.dat(w, d),
            )
        };
        self.out.code[*self.code_cache.get(&f).unwrap()].values[i] = v;
        return i;
    }
    pub fn func<W: MTBehavior<T, Y, R, D, S, L, E, P>>(
        &mut self,
        w: &mut W,
        f: Id<Fun<T, Y, R, D>>,
    ) -> Id<Fun<S, L, E, P>> {
        if let Some(i) = self.code_cache.get(&f) {
            return *i;
        }
        let me = self.out.code.alloc(Default::default());
        let m = Rc::new(RefCell::new(BTreeMap::new()));
        // return self.out.code.alloc_with_id(|me| {
        self.code_cache.insert(f.clone(), me.clone());
        let a = &self.input.code[f.clone()];
        let t = w.terminator(
            self,
            FuncTransformCtx {
                input: f.clone(),
                output: me.clone(),
            },
            self.input.code[f].terminator.clone(),
            |w, t, v| t.func_value(w, f, m.clone(), v),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        );
        self.out.code[me].terminator = t;
        return me;
        // });
    }
    pub fn dat<W: MTBehavior<T, Y, R, D, S, L, E, P>>(&mut self, w: &mut W, dd: Id<D>) -> Id<P> {
        if let Some(b) = self.datum_cache.get(&dd) {
            return *b;
        }
        let d = w.datum(
            self,
            self.input.data[dd].clone(),
            |w, t, f| t.func(w, f),
            |w, t, d| t.dat(w, d),
        );
        let e = self.out.data.alloc(d);
        self.datum_cache.insert(dd, e);
        return e;
    }
}