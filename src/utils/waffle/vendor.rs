use std::{convert::Infallible, borrow::Cow};

use waffle::*;
pub fn op_outputs(
    module: &Module,
    op_stack: &[(Type, Value)],
    op: &Operator,
) -> Result<Cow<'static, [Type]>,Infallible> {
    match op {
        &Operator::Unreachable | &Operator::Nop => Ok(Cow::Borrowed(&[])),

        &Operator::Call { function_index } => {
            let sig = module.funcs[function_index].sig();
            Ok(Vec::from(module.signatures[sig].returns.clone()).into())
        }
        &Operator::CallIndirect { sig_index, .. } => {
            Ok(Vec::from(module.signatures[sig_index].returns.clone()).into())
        }

        &Operator::Select => {
            let val_ty = op_stack[op_stack.len() - 2].0;
            Ok(vec![val_ty].into())
        }
        &Operator::TypedSelect { ty } => Ok(vec![ty].into()),
        &Operator::GlobalGet { global_index } => Ok(vec![module.globals[global_index].ty].into()),
        &Operator::GlobalSet { .. } => Ok(Cow::Borrowed(&[])),

        Operator::I32Load { .. }
        | Operator::I32Load8S { .. }
        | Operator::I32Load8U { .. }
        | Operator::I32Load16S { .. }
        | Operator::I32Load16U { .. } => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I64Load { .. }
        | Operator::I64Load8S { .. }
        | Operator::I64Load8U { .. }
        | Operator::I64Load16S { .. }
        | Operator::I64Load16U { .. }
        | Operator::I64Load32S { .. }
        | Operator::I64Load32U { .. } => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::F32Load { .. } => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F64Load { .. } => Ok(Cow::Borrowed(&[Type::F64])),

        Operator::I32Store { .. } => Ok(Cow::Borrowed(&[])),
        Operator::I64Store { .. } => Ok(Cow::Borrowed(&[])),
        Operator::F32Store { .. } => Ok(Cow::Borrowed(&[])),
        Operator::F64Store { .. } => Ok(Cow::Borrowed(&[])),
        Operator::I32Store8 { .. } => Ok(Cow::Borrowed(&[])),
        Operator::I32Store16 { .. } => Ok(Cow::Borrowed(&[])),
        Operator::I64Store8 { .. } => Ok(Cow::Borrowed(&[])),
        Operator::I64Store16 { .. } => Ok(Cow::Borrowed(&[])),
        Operator::I64Store32 { .. } => Ok(Cow::Borrowed(&[])),

        Operator::I32Const { .. } => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I64Const { .. } => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::F32Const { .. } => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F64Const { .. } => Ok(Cow::Borrowed(&[Type::F64])),

        Operator::I32Eqz
        | Operator::I32Eq
        | Operator::I32Ne
        | Operator::I32LtS
        | Operator::I32LtU
        | Operator::I32GtS
        | Operator::I32GtU
        | Operator::I32LeS
        | Operator::I32LeU
        | Operator::I32GeS
        | Operator::I32GeU
        | Operator::I64Eqz
        | Operator::I64Eq
        | Operator::I64Ne
        | Operator::I64LtS
        | Operator::I64LtU
        | Operator::I64GtU
        | Operator::I64GtS
        | Operator::I64LeS
        | Operator::I64LeU
        | Operator::I64GeS
        | Operator::I64GeU
        | Operator::F32Eq
        | Operator::F32Ne
        | Operator::F32Lt
        | Operator::F32Gt
        | Operator::F32Le
        | Operator::F32Ge
        | Operator::F64Eq
        | Operator::F64Ne
        | Operator::F64Lt
        | Operator::F64Gt
        | Operator::F64Le
        | Operator::F64Ge => Ok(Cow::Borrowed(&[Type::I32])),

        Operator::I32Clz
        | Operator::I32Ctz
        | Operator::I32Popcnt
        | Operator::I32Add
        | Operator::I32Sub
        | Operator::I32Mul
        | Operator::I32DivS
        | Operator::I32DivU
        | Operator::I32RemS
        | Operator::I32RemU
        | Operator::I32And
        | Operator::I32Or
        | Operator::I32Xor
        | Operator::I32Shl
        | Operator::I32ShrS
        | Operator::I32ShrU
        | Operator::I32Rotl
        | Operator::I32Rotr => Ok(Cow::Borrowed(&[Type::I32])),

        Operator::I64Clz
        | Operator::I64Ctz
        | Operator::I64Popcnt
        | Operator::I64Add
        | Operator::I64Sub
        | Operator::I64Mul
        | Operator::I64DivS
        | Operator::I64DivU
        | Operator::I64RemS
        | Operator::I64RemU
        | Operator::I64And
        | Operator::I64Or
        | Operator::I64Xor
        | Operator::I64Shl
        | Operator::I64ShrS
        | Operator::I64ShrU
        | Operator::I64Rotl
        | Operator::I64Rotr => Ok(Cow::Borrowed(&[Type::I64])),

        Operator::F32Abs
        | Operator::F32Neg
        | Operator::F32Ceil
        | Operator::F32Floor
        | Operator::F32Trunc
        | Operator::F32Nearest
        | Operator::F32Sqrt
        | Operator::F32Add
        | Operator::F32Sub
        | Operator::F32Mul
        | Operator::F32Div
        | Operator::F32Min
        | Operator::F32Max
        | Operator::F32Copysign => Ok(Cow::Borrowed(&[Type::F32])),

        Operator::F64Abs
        | Operator::F64Neg
        | Operator::F64Ceil
        | Operator::F64Floor
        | Operator::F64Trunc
        | Operator::F64Nearest
        | Operator::F64Sqrt
        | Operator::F64Add
        | Operator::F64Sub
        | Operator::F64Mul
        | Operator::F64Div
        | Operator::F64Min
        | Operator::F64Max
        | Operator::F64Copysign => Ok(Cow::Borrowed(&[Type::F64])),

        Operator::I32WrapI64 => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncF32S => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncF32U => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncF64S => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncF64U => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I64ExtendI32S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64ExtendI32U => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncF32S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncF32U => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncF64S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncF64U => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::F32ConvertI32S => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F32ConvertI32U => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F32ConvertI64S => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F32ConvertI64U => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F32DemoteF64 => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F64ConvertI32S => Ok(Cow::Borrowed(&[Type::F64])),
        Operator::F64ConvertI32U => Ok(Cow::Borrowed(&[Type::F64])),
        Operator::F64ConvertI64S => Ok(Cow::Borrowed(&[Type::F64])),
        Operator::F64ConvertI64U => Ok(Cow::Borrowed(&[Type::F64])),
        Operator::F64PromoteF32 => Ok(Cow::Borrowed(&[Type::F64])),
        Operator::I32Extend8S => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32Extend16S => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I64Extend8S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64Extend16S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64Extend32S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I32TruncSatF32S => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncSatF32U => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncSatF64S => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I32TruncSatF64U => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I64TruncSatF32S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncSatF32U => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncSatF64S => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::I64TruncSatF64U => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::F32ReinterpretI32 => Ok(Cow::Borrowed(&[Type::F32])),
        Operator::F64ReinterpretI64 => Ok(Cow::Borrowed(&[Type::F64])),
        Operator::I32ReinterpretF32 => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::I64ReinterpretF64 => Ok(Cow::Borrowed(&[Type::I64])),
        Operator::TableGet { table_index } => Ok(vec![module.tables[*table_index].ty].into()),
        Operator::TableSet { .. } => Ok(Cow::Borrowed(&[])),
        Operator::TableGrow { .. } => Ok(Cow::Borrowed(&[])),
        Operator::TableSize { .. } => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::MemorySize { .. } => Ok(Cow::Borrowed(&[Type::I32])),
        Operator::MemoryGrow { .. } => Ok(Cow::Borrowed(&[Type::I32])),
    }
}
