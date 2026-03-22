// SPDX-License-Identifier: MIT
//! LLVM Code Generation for My Language
//!
//! This crate uses inkwell to generate LLVM IR from MIR, producing
//! native binaries for multiple targets.
//!
//! # Targets
//!
//! - x86_64-linux (primary)
//! - x86_64-darwin (macOS Intel)
//! - aarch64-darwin (macOS Apple Silicon)
//! - x86_64-windows
//! - wasm32-unknown
//! - aarch64-linux

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use my_mir::{BasicBlock, BinOp, Instruction, InstructionKind, MirFunction, MirProgram, MirType};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Code generation errors
#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("LLVM error: {0}")]
    LlvmError(String),

    #[error("unsupported target: {0}")]
    UnsupportedTarget(String),

    #[error("undefined function: {0}")]
    UndefinedFunction(String),

    #[error("type error: {0}")]
    TypeError(String),
}

/// Target triple specification
#[derive(Debug, Clone)]
pub struct TargetSpec {
    pub triple: String,
    pub cpu: String,
    pub features: String,
}

impl TargetSpec {
    pub fn host() -> Self {
        TargetSpec {
            triple: TargetMachine::get_default_triple().to_string(),
            cpu: TargetMachine::get_host_cpu_name().to_string(),
            features: TargetMachine::get_host_cpu_features().to_string(),
        }
    }

    pub fn x86_64_linux() -> Self {
        TargetSpec {
            triple: "x86_64-unknown-linux-gnu".to_string(),
            cpu: "x86-64".to_string(),
            features: String::new(),
        }
    }

    pub fn aarch64_darwin() -> Self {
        TargetSpec {
            triple: "aarch64-apple-darwin".to_string(),
            cpu: "apple-m1".to_string(),
            features: String::new(),
        }
    }

    pub fn wasm32() -> Self {
        TargetSpec {
            triple: "wasm32-unknown-unknown".to_string(),
            cpu: "generic".to_string(),
            features: String::new(),
        }
    }
}

/// LLVM code generator
pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    values: HashMap<my_mir::LocalId, BasicValueEnum<'ctx>>,
    target: TargetSpec,
}

impl<'ctx> Codegen<'ctx> {
    /// Create a new code generator
    pub fn new(context: &'ctx Context, name: &str, target: TargetSpec) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();

        // Set target triple
        module.set_triple(&inkwell::targets::TargetTriple::create(&target.triple));

        Codegen {
            context,
            module,
            builder,
            functions: HashMap::new(),
            values: HashMap::new(),
            target,
        }
    }

    /// Generate LLVM IR from MIR program
    pub fn generate(&mut self, program: &MirProgram) -> Result<(), CodegenError> {
        // First pass: declare all functions
        for (name, func) in &program.functions {
            let fn_value = self.declare_function(func)?;
            self.functions.insert(name.clone(), fn_value);
        }

        // Second pass: generate function bodies
        for (name, func) in &program.functions {
            let fn_value = *self
                .functions
                .get(name)
                .ok_or_else(|| CodegenError::UndefinedFunction(name.clone()))?;
            self.generate_function(func, fn_value)?;
        }

        Ok(())
    }

    /// Declare a function (for forward references)
    fn declare_function(&self, func: &MirFunction) -> Result<FunctionValue<'ctx>, CodegenError> {
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.lower_type(&p.ty).into())
            .collect();

        let fn_type = match &func.return_type {
            MirType::Unit | MirType::Never => {
                self.context.void_type().fn_type(&param_types, false)
            }
            ret => self.lower_type(ret).fn_type(&param_types, false),
        };

        Ok(self.module.add_function(&func.name, fn_type, None))
    }

    /// Generate a function body
    fn generate_function(
        &mut self,
        func: &MirFunction,
        fn_value: FunctionValue<'ctx>,
    ) -> Result<(), CodegenError> {
        use inkwell::basic_block::BasicBlock as LLVMBasicBlock;
        use inkwell::IntPredicate;

        // Clear values for new function
        self.values.clear();

        // Bind parameters to locals
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
            self.values.insert(param.id, param_value);
        }

        // First pass: create all LLVM basic blocks
        let mut llvm_blocks: HashMap<my_mir::BlockId, LLVMBasicBlock> = HashMap::new();
        for node_idx in func.blocks.node_indices() {
            let mir_block = func.blocks.node_weight(node_idx).unwrap();
            let llvm_block = self.context.append_basic_block(fn_value, &format!("bb{}", mir_block.id.0));
            llvm_blocks.insert(mir_block.id, llvm_block);
        }

        // Second pass: generate instructions for each block
        for node_idx in func.blocks.node_indices() {
            let mir_block = func.blocks.node_weight(node_idx).unwrap().clone();
            let llvm_block = llvm_blocks[&mir_block.id];
            self.builder.position_at_end(llvm_block);

            // Generate instructions
            for instr in &mir_block.instructions {
                let value = self.generate_instruction(instr, fn_value)?;
                self.values.insert(instr.dest, value);
            }

            // Generate terminator
            self.generate_terminator(&mir_block.terminator, &llvm_blocks)?;
        }

        Ok(())
    }

    /// Generate a single instruction
    fn generate_instruction(
        &mut self,
        instr: &Instruction,
        fn_value: FunctionValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match &instr.kind {
            InstructionKind::Const(c) => Ok(self.generate_constant(c)),

            InstructionKind::BinOp(op, left, right) => {
                let lhs = self.get_value(*left)?;
                let rhs = self.get_value(*right)?;
                self.generate_binop(*op, lhs, rhs)
            }

            InstructionKind::UnOp(op, operand) => {
                let val = self.get_value(*operand)?;
                self.generate_unop(*op, val)
            }

            InstructionKind::Call(name, args) => {
                let callee = self.functions.get(name)
                    .ok_or_else(|| CodegenError::UndefinedFunction(name.clone()))?;

                let arg_values: Result<Vec<_>, _> = args.iter()
                    .map(|&arg| self.get_value(arg).map(Into::into))
                    .collect();

                let call_site = self.builder.build_call(*callee, &arg_values?, "call")
                    .map_err(|e| CodegenError::LlvmError(format!("build_call failed: {:?}", e)))?;

                match call_site.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(val) => Ok(val),
                    inkwell::values::ValueKind::Instruction(_) => {
                        Ok(self.context.i8_type().const_int(0, false).into())
                    }
                }
            }

            InstructionKind::CallIndirect(func_ptr, args) => {
                let fn_ptr = self.get_value(*func_ptr)?;

                if let BasicValueEnum::PointerValue(ptr) = fn_ptr {
                    let arg_values: Result<Vec<_>, _> = args.iter()
                        .map(|&arg| self.get_value(arg).map(Into::into))
                        .collect();

                    // For indirect calls, we need the function type
                    // Create a generic function pointer type
                    let void_fn_ty = self.context.void_type().fn_type(&[], false);

                    let call_site = self.builder.build_indirect_call(
                        void_fn_ty,
                        ptr,
                        &arg_values?,
                        "indirect_call"
                    ).map_err(|e| CodegenError::LlvmError(format!("build_indirect_call failed: {:?}", e)))?;

                    match call_site.try_as_basic_value() {
                        inkwell::values::ValueKind::Basic(val) => Ok(val),
                        inkwell::values::ValueKind::Instruction(_) => {
                            Ok(self.context.i8_type().const_int(0, false).into())
                        }
                    }
                } else {
                    Err(CodegenError::TypeError("expected function pointer for indirect call".to_string()))
                }
            }

            InstructionKind::Load(ptr) => {
                let ptr_val = self.get_value(*ptr)?;
                if let BasicValueEnum::PointerValue(ptr) = ptr_val {
                    let loaded = self.builder.build_load(self.context.i64_type(), ptr, "load")
                        .map_err(|e| CodegenError::LlvmError(format!("build_load failed: {:?}", e)))?;
                    Ok(loaded)
                } else {
                    Err(CodegenError::TypeError("expected pointer type".to_string()))
                }
            }

            InstructionKind::Store(ptr, val) => {
                let ptr_val = self.get_value(*ptr)?;
                let value = self.get_value(*val)?;
                if let BasicValueEnum::PointerValue(ptr) = ptr_val {
                    self.builder.build_store(ptr, value)
                        .map_err(|e| CodegenError::LlvmError(format!("build_store failed: {:?}", e)))?;
                }
                Ok(self.context.i8_type().const_int(0, false).into())
            }

            InstructionKind::Alloca(ty) => {
                let llvm_ty = self.lower_type(ty);
                let ptr = self.builder.build_alloca(llvm_ty, "alloca")
                    .map_err(|e| CodegenError::LlvmError(format!("build_alloca failed: {:?}", e)))?;
                Ok(ptr.into())
            }

            InstructionKind::GetElementPtr(base, indices) => {
                let base_val = self.get_value(*base)?;
                if let BasicValueEnum::PointerValue(ptr) = base_val {
                    let idx_values: Result<Vec<_>, _> = indices.iter()
                        .map(|&idx| {
                            self.get_value(idx).map(|v| v.into_int_value())
                        })
                        .collect();

                    unsafe {
                        let gep = self.builder.build_gep(
                            self.context.i8_type(),
                            ptr,
                            &idx_values?,
                            "gep"
                        ).map_err(|e| CodegenError::LlvmError(format!("build_gep failed: {:?}", e)))?;
                        Ok(gep.into())
                    }
                } else {
                    Err(CodegenError::TypeError("expected pointer for GEP".to_string()))
                }
            }

            InstructionKind::Cast(val, ty) => {
                let value = self.get_value(*val)?;
                let target_ty = self.lower_type(ty);

                // Perform appropriate cast based on types
                match (value, target_ty) {
                    // Int to int
                    (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(target_int_ty)) => {
                        let cast = self.builder.build_int_cast(int_val, target_int_ty, "cast")
                            .map_err(|e| CodegenError::LlvmError(format!("int_cast failed: {:?}", e)))?;
                        Ok(cast.into())
                    }
                    // Float to int
                    (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::IntType(target_int_ty)) => {
                        let cast = self.builder.build_float_to_signed_int(float_val, target_int_ty, "ftoi")
                            .map_err(|e| CodegenError::LlvmError(format!("float_to_int failed: {:?}", e)))?;
                        Ok(cast.into())
                    }
                    // Int to float
                    (BasicValueEnum::IntValue(int_val), BasicTypeEnum::FloatType(target_float_ty)) => {
                        let cast = self.builder.build_signed_int_to_float(int_val, target_float_ty, "itof")
                            .map_err(|e| CodegenError::LlvmError(format!("int_to_float failed: {:?}", e)))?;
                        Ok(cast.into())
                    }
                    // Float to float
                    (BasicValueEnum::FloatValue(float_val), BasicTypeEnum::FloatType(target_float_ty)) => {
                        let cast = self.builder.build_float_cast(float_val, target_float_ty, "fcast")
                            .map_err(|e| CodegenError::LlvmError(format!("float_cast failed: {:?}", e)))?;
                        Ok(cast.into())
                    }
                    // Pointer casts
                    (BasicValueEnum::PointerValue(ptr_val), BasicTypeEnum::PointerType(target_ptr_ty)) => {
                        let cast = self.builder.build_pointer_cast(ptr_val, target_ptr_ty, "ptrcast")
                            .map_err(|e| CodegenError::LlvmError(format!("pointer_cast failed: {:?}", e)))?;
                        Ok(cast.into())
                    }
                    // Bitcast for other types
                    _ => {
                        let cast = self.builder.build_bit_cast(value, target_ty, "bitcast")
                            .map_err(|e| CodegenError::LlvmError(format!("bitcast failed: {:?}", e)))?;
                        Ok(cast)
                    }
                }
            }

            InstructionKind::Phi(branches) => {
                // Create a PHI node
                if branches.is_empty() {
                    return Ok(self.context.i8_type().const_int(0, false).into());
                }

                // Get the type from the first incoming value
                let (_, first_local) = branches[0];
                let first_value = self.get_value(first_local)?;
                let phi_type = first_value.get_type();

                // Create PHI node
                let phi = self.builder.build_phi(phi_type, "phi")
                    .map_err(|e| CodegenError::LlvmError(format!("build_phi failed: {:?}", e)))?;

                // Note: Incoming values need to be added after all blocks are generated
                // For now, return the first value as a fallback
                // In a complete implementation, we'd track PHI nodes and populate them
                // in a final pass after all blocks are complete
                Ok(first_value)
            }

            InstructionKind::AIStub(op, args) => {
                self.generate_ai_stub(op, args)
            }

            InstructionKind::Drop(_) => {
                Ok(self.context.i8_type().const_int(0, false).into())
            }
            InstructionKind::Copy(src) | InstructionKind::Move(src) => {
                self.get_value(*src)
            }
        }
    }

    /// Generate a block terminator
    fn generate_terminator(
        &mut self,
        terminator: &my_mir::Terminator,
        blocks: &HashMap<my_mir::BlockId, inkwell::basic_block::BasicBlock<'ctx>>,
    ) -> Result<(), CodegenError> {
        use my_mir::Terminator;

        match terminator {
            Terminator::Return(val) => {
                if let Some(local) = val {
                    let ret_val = self.get_value(*local)?;
                    self.builder.build_return(Some(&ret_val))
                        .map_err(|e| CodegenError::LlvmError(format!("build_return failed: {:?}", e)))?;
                } else {
                    self.builder.build_return(None)
                        .map_err(|e| CodegenError::LlvmError(format!("build_return failed: {:?}", e)))?;
                }
            }

            Terminator::Goto(target) => {
                let target_block = blocks.get(target)
                    .ok_or_else(|| CodegenError::TypeError(format!("undefined block {:?}", target)))?;
                self.builder.build_unconditional_branch(*target_block)
                    .map_err(|e| CodegenError::LlvmError(format!("build_unconditional_branch failed: {:?}", e)))?;
            }

            Terminator::If(cond, then_block, else_block) => {
                let cond_val = self.get_value(*cond)?;
                let cond_int = cond_val.into_int_value();

                let then_bb = blocks.get(then_block)
                    .ok_or_else(|| CodegenError::TypeError(format!("undefined block {:?}", then_block)))?;
                let else_bb = blocks.get(else_block)
                    .ok_or_else(|| CodegenError::TypeError(format!("undefined block {:?}", else_block)))?;

                self.builder.build_conditional_branch(cond_int, *then_bb, *else_bb)
                    .map_err(|e| CodegenError::LlvmError(format!("build_conditional_branch failed: {:?}", e)))?;
            }

            Terminator::Switch(val, cases, default) => {
                let switch_val = self.get_value(*val)?;
                let switch_int = switch_val.into_int_value();

                let default_bb = blocks.get(default)
                    .ok_or_else(|| CodegenError::TypeError(format!("undefined block {:?}", default)))?;

                let llvm_cases: Vec<_> = cases.iter()
                    .filter_map(|(val, block_id)| {
                        blocks.get(block_id).map(|bb| {
                            (self.context.i64_type().const_int(*val as u64, false), *bb)
                        })
                    })
                    .collect();

                self.builder.build_switch(switch_int, *default_bb, &llvm_cases)
                    .map_err(|e| CodegenError::LlvmError(format!("build_switch failed: {:?}", e)))?;
            }

            Terminator::Unreachable => {
                self.builder.build_unreachable()
                    .map_err(|e| CodegenError::LlvmError(format!("build_unreachable failed: {:?}", e)))?;
            }

            Terminator::Invoke { func, args, dest, normal, unwind } => {
                // Invoke is for exception handling - simplified for now
                let callee = self.functions.get(func)
                    .ok_or_else(|| CodegenError::UndefinedFunction(func.clone()))?;

                let arg_values: Result<Vec<_>, _> = args.iter()
                    .map(|&arg| self.get_value(arg).map(Into::into))
                    .collect();

                let call_site = self.builder.build_call(*callee, &arg_values?, "invoke")
                    .map_err(|e| CodegenError::LlvmError(format!("build_call failed: {:?}", e)))?;

                if let inkwell::values::ValueKind::Basic(ret_val) = call_site.try_as_basic_value() {
                    self.values.insert(*dest, ret_val);
                }

                let normal_bb = blocks.get(normal)
                    .ok_or_else(|| CodegenError::TypeError(format!("undefined block {:?}", normal)))?;

                self.builder.build_unconditional_branch(*normal_bb)
                    .map_err(|e| CodegenError::LlvmError(format!("build_unconditional_branch failed: {:?}", e)))?;
            }
        }

        Ok(())
    }

    /// Get a value from the values map
    fn get_value(&self, local: my_mir::LocalId) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        self.values.get(&local).copied()
            .ok_or_else(|| CodegenError::TypeError(format!("undefined local {:?}", local)))
    }

    /// Generate a constant value
    fn generate_constant(&self, c: &my_mir::MirConstant) -> BasicValueEnum<'ctx> {
        use my_mir::MirConstant;

        match c {
            MirConstant::I32(v) => self.context.i32_type().const_int(*v as u64, false).into(),
            MirConstant::I64(v) => self.context.i64_type().const_int(*v as u64, false).into(),
            MirConstant::F32(v) => self.context.f32_type().const_float(*v as f64).into(),
            MirConstant::F64(v) => self.context.f64_type().const_float(*v).into(),
            MirConstant::Bool(v) => self.context.bool_type().const_int(if *v { 1 } else { 0 }, false).into(),
            MirConstant::String(s) => {
                // Create a global string constant
                let str_val = self.context.const_string(s.as_bytes(), true);
                let global = self.module.add_global(str_val.get_type(), None, "str");
                global.set_initializer(&str_val);
                global.as_pointer_value().into()
            }
            MirConstant::Unit => self.context.i8_type().const_int(0, false).into(),
        }
    }

    /// Generate a binary operation
    fn generate_binop(
        &self,
        op: my_mir::BinOp,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        use my_mir::BinOp;
        use inkwell::IntPredicate;

        // Try integer operations first
        if lhs.is_int_value() && rhs.is_int_value() {
            let (l, r) = (lhs.into_int_value(), rhs.into_int_value());
            let result = match op {
                BinOp::Add => self.builder.build_int_add(l, r, "add"),
                BinOp::Sub => self.builder.build_int_sub(l, r, "sub"),
                BinOp::Mul => self.builder.build_int_mul(l, r, "mul"),
                BinOp::Div => self.builder.build_int_signed_div(l, r, "div"),
                BinOp::Rem => self.builder.build_int_signed_rem(l, r, "rem"),
                BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, l, r, "eq"),
                BinOp::Ne => self.builder.build_int_compare(IntPredicate::NE, l, r, "ne"),
                BinOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, l, r, "lt"),
                BinOp::Le => self.builder.build_int_compare(IntPredicate::SLE, l, r, "le"),
                BinOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, l, r, "gt"),
                BinOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, l, r, "ge"),
                BinOp::And => self.builder.build_and(l, r, "and"),
                BinOp::Or => self.builder.build_or(l, r, "or"),
                BinOp::Xor => self.builder.build_xor(l, r, "xor"),
                BinOp::Shl => self.builder.build_left_shift(l, r, "shl"),
                BinOp::Shr => self.builder.build_right_shift(l, r, true, "shr"),
            }.map_err(|e| CodegenError::LlvmError(format!("binop failed: {:?}", e)))?;
            return Ok(result.into());
        }

        // Try float operations
        if lhs.is_float_value() && rhs.is_float_value() {
            let (l, r) = (lhs.into_float_value(), rhs.into_float_value());
            // Float comparisons return IntValue (i1), arithmetic returns FloatValue
            match op {
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    let pred = match op {
                        BinOp::Eq => inkwell::FloatPredicate::OEQ,
                        BinOp::Ne => inkwell::FloatPredicate::ONE,
                        BinOp::Lt => inkwell::FloatPredicate::OLT,
                        BinOp::Le => inkwell::FloatPredicate::OLE,
                        BinOp::Gt => inkwell::FloatPredicate::OGT,
                        BinOp::Ge => inkwell::FloatPredicate::OGE,
                        _ => unreachable!(),
                    };
                    let cmp = self.builder.build_float_compare(pred, l, r, "fcmp")
                        .map_err(|e| CodegenError::LlvmError(format!("float cmp failed: {:?}", e)))?;
                    return Ok(cmp.into());
                }
                _ => {
                    let result = match op {
                        BinOp::Add => self.builder.build_float_add(l, r, "fadd"),
                        BinOp::Sub => self.builder.build_float_sub(l, r, "fsub"),
                        BinOp::Mul => self.builder.build_float_mul(l, r, "fmul"),
                        BinOp::Div => self.builder.build_float_div(l, r, "fdiv"),
                        BinOp::Rem => self.builder.build_float_rem(l, r, "frem"),
                        _ => return Err(CodegenError::TypeError("invalid float operation".to_string())),
                    }.map_err(|e| CodegenError::LlvmError(format!("float binop failed: {:?}", e)))?;
                    return Ok(result.into());
                }
            }
        }

        Err(CodegenError::TypeError("type mismatch in binary operation".to_string()))
    }

    /// Generate a unary operation
    fn generate_unop(
        &self,
        op: my_mir::UnOp,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        use my_mir::UnOp;

        match op {
            UnOp::Neg => {
                if val.is_int_value() {
                    Ok(self.builder.build_int_neg(val.into_int_value(), "neg")
                        .map_err(|e| CodegenError::LlvmError(format!("int_neg failed: {:?}", e)))?
                        .into())
                } else if val.is_float_value() {
                    Ok(self.builder.build_float_neg(val.into_float_value(), "fneg")
                        .map_err(|e| CodegenError::LlvmError(format!("float_neg failed: {:?}", e)))?
                        .into())
                } else {
                    Err(CodegenError::TypeError("invalid type for negation".to_string()))
                }
            }
            UnOp::Not => {
                if val.is_int_value() {
                    Ok(self.builder.build_not(val.into_int_value(), "not")
                        .map_err(|e| CodegenError::LlvmError(format!("not failed: {:?}", e)))?
                        .into())
                } else {
                    Err(CodegenError::TypeError("expected integer for not".to_string()))
                }
            }
            UnOp::Deref => {
                // Deref is handled by Load instruction
                Ok(val)
            }
            UnOp::AddrOf | UnOp::AddrOfMut => {
                // Address-of is handled by alloca
                Ok(val)
            }
        }
    }

    /// Generate an AI runtime stub call
    fn generate_ai_stub(
        &mut self,
        op: &my_mir::AIOperation,
        args: &[my_mir::LocalId],
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        use my_mir::AIOperation;

        let arg_values: Result<Vec<_>, _> = args.iter()
            .map(|&arg| self.get_value(arg).map(Into::into))
            .collect();
        let arg_vals = arg_values?;

        let null_ptr = self.context.ptr_type(inkwell::AddressSpace::default()).const_null().into();

        let extract_call = |call_site: inkwell::values::CallSiteValue<'ctx>, default: BasicValueEnum<'ctx>| -> BasicValueEnum<'ctx> {
            match call_site.try_as_basic_value() {
                inkwell::values::ValueKind::Basic(val) => val,
                inkwell::values::ValueKind::Instruction(_) => default,
            }
        };

        match op {
            AIOperation::Query { model: _ } => {
                let func_name = "my_ai_query";
                if let Some(func) = self.module.get_function(func_name) {
                    let call_site = self.builder.build_call(func, &arg_vals, "ai_query")
                        .map_err(|e| CodegenError::LlvmError(format!("ai_query call failed: {:?}", e)))?;
                    Ok(extract_call(call_site, self.context.i8_type().const_int(0, false).into()))
                } else {
                    Ok(null_ptr)
                }
            }

            AIOperation::Verify => {
                let func_name = "my_ai_verify";
                if let Some(func) = self.module.get_function(func_name) {
                    let call_site = self.builder.build_call(func, &arg_vals, "ai_verify")
                        .map_err(|e| CodegenError::LlvmError(format!("ai_verify call failed: {:?}", e)))?;
                    Ok(extract_call(call_site, self.context.bool_type().const_int(0, false).into()))
                } else {
                    Ok(self.context.bool_type().const_int(0, false).into())
                }
            }

            AIOperation::Embed => {
                let func_name = "my_ai_embed";
                if let Some(func) = self.module.get_function(func_name) {
                    let call_site = self.builder.build_call(func, &arg_vals, "ai_embed")
                        .map_err(|e| CodegenError::LlvmError(format!("ai_embed call failed: {:?}", e)))?;
                    Ok(extract_call(call_site, null_ptr))
                } else {
                    Ok(null_ptr)
                }
            }

            AIOperation::Generate | AIOperation::Classify => {
                let func_name = "my_ai_query";
                if let Some(func) = self.module.get_function(func_name) {
                    let call_site = self.builder.build_call(func, &arg_vals, "ai_generate")
                        .map_err(|e| CodegenError::LlvmError(format!("ai_generate call failed: {:?}", e)))?;
                    Ok(extract_call(call_site, null_ptr))
                } else {
                    Ok(null_ptr)
                }
            }
        }
    }

    /// Lower MIR type to LLVM type
    fn lower_type(&self, ty: &MirType) -> BasicTypeEnum<'ctx> {
        match ty {
            MirType::I32 => self.context.i32_type().into(),
            MirType::I64 => self.context.i64_type().into(),
            MirType::F32 => self.context.f32_type().into(),
            MirType::F64 => self.context.f64_type().into(),
            MirType::Bool => self.context.bool_type().into(),
            MirType::Ptr(_inner) => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            MirType::Array(inner, size) => self.lower_type(inner).array_type(*size as u32).into(),
            MirType::Struct(name, fields) => {
                let field_types: Vec<BasicTypeEnum> =
                    fields.iter().map(|f| self.lower_type(f)).collect();
                self.context.struct_type(&field_types, false).into()
            }
            MirType::Unit => self.context.i8_type().into(), // Unit as i8
            MirType::Never => self.context.i8_type().into(),
            MirType::Function(_, _) => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        }
    }

    /// Get the LLVM module
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Write LLVM IR to file
    pub fn write_ir(&self, path: &Path) -> Result<(), CodegenError> {
        self.module
            .print_to_file(path)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Write bitcode to file
    pub fn write_bitcode(&self, path: &Path) -> bool {
        self.module.write_bitcode_to_path(path)
    }

    /// Compile to object file
    pub fn compile_to_object(&self, path: &Path) -> Result<(), CodegenError> {
        Target::initialize_all(&InitializationConfig::default());

        let target = Target::from_triple(&inkwell::targets::TargetTriple::create(&self.target.triple))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let target_machine = target
            .create_target_machine(
                &inkwell::targets::TargetTriple::create(&self.target.triple),
                &self.target.cpu,
                &self.target.features,
                OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| CodegenError::UnsupportedTarget(self.target.triple.clone()))?;

        target_machine
            .write_to_file(&self.module, FileType::Object, path)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Compile to a native executable by linking the object file
    pub fn compile_to_binary(&self, output: &Path) -> Result<(), CodegenError> {
        use std::process::Command;

        // Create a temp directory for the intermediate object file
        let obj_path = output.with_extension("o");

        // Generate the object file
        self.compile_to_object(&obj_path)?;

        // Link using the system C compiler (cc) which handles platform-specific linking
        let status = Command::new("cc")
            .arg(&obj_path)
            .arg("-o")
            .arg(output)
            .arg("-lm") // Link math library
            .status()
            .map_err(|e| {
                CodegenError::LlvmError(format!(
                    "failed to invoke linker 'cc': {} (is a C compiler installed?)",
                    e
                ))
            })?;

        // Clean up the intermediate object file
        let _ = std::fs::remove_file(&obj_path);

        if !status.success() {
            return Err(CodegenError::LlvmError(format!(
                "linker failed with exit code: {}",
                status.code().unwrap_or(-1)
            )));
        }

        Ok(())
    }

    /// Run optimization passes using the new LLVM pass manager
    pub fn optimize(&self, level: OptLevel) -> Result<(), CodegenError> {
        Target::initialize_all(&InitializationConfig::default());

        let target = Target::from_triple(&inkwell::targets::TargetTriple::create(&self.target.triple))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let target_machine = target
            .create_target_machine(
                &inkwell::targets::TargetTriple::create(&self.target.triple),
                &self.target.cpu,
                &self.target.features,
                match level {
                    OptLevel::None => OptimizationLevel::None,
                    OptLevel::Less => OptimizationLevel::Less,
                    OptLevel::Default => OptimizationLevel::Default,
                    OptLevel::Aggressive => OptimizationLevel::Aggressive,
                },
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| CodegenError::UnsupportedTarget(self.target.triple.clone()))?;

        let passes = match level {
            OptLevel::None => "default<O0>",
            OptLevel::Less => "default<O1>",
            OptLevel::Default => "default<O2>",
            OptLevel::Aggressive => "default<O3>",
        };

        self.module
            .run_passes(
                passes,
                &target_machine,
                inkwell::passes::PassBuilderOptions::create(),
            )
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Verify the module
    pub fn verify(&self) -> Result<(), CodegenError> {
        self.module
            .verify()
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }
}

/// Optimization level
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    None,
    Less,
    Default,
    Aggressive,
}

/// Re-export inkwell Context for callers that need fine-grained control
pub use inkwell::context::Context as LlvmContext;

/// High-level compile-to-binary pipeline.
///
/// This is a convenience function that creates the LLVM context, codegen,
/// runs optimization, verifies the IR, and links to a native executable.
pub fn compile_program(
    mir: &MirProgram,
    source_name: &str,
    output: &Path,
    opt_level: OptLevel,
) -> Result<(), CodegenError> {
    let context = Context::create();
    let target = TargetSpec::host();
    let mut codegen = Codegen::new(&context, source_name, target);

    // Declare runtime functions
    runtime::declare_runtime(&mut codegen);
    ai_stubs::generate_stubs(&mut codegen);

    codegen.generate(mir)?;
    codegen.optimize(opt_level)?;
    codegen.verify()?;
    codegen.compile_to_binary(output)?;

    Ok(())
}

/// AI runtime stub generator
pub mod ai_stubs {
    use super::*;

    /// Generate AI runtime stubs for linking
    pub fn generate_stubs<'ctx>(codegen: &mut Codegen<'ctx>) {
        // Declare external AI runtime functions
        let ptr_ty = codegen.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = codegen.context.i64_type();

        // my_ai_query(model: *const i8, prompt: *const i8) -> *const i8
        let query_ty = ptr_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        codegen.module.add_function("my_ai_query", query_ty, None);

        // my_ai_verify(condition: *const i8) -> i64 (bool)
        let verify_ty = i64_ty.fn_type(&[ptr_ty.into()], false);
        codegen.module.add_function("my_ai_verify", verify_ty, None);

        // my_ai_embed(text: *const i8) -> *const f32
        let embed_ty = ptr_ty.fn_type(&[ptr_ty.into()], false);
        codegen.module.add_function("my_ai_embed", embed_ty, None);
    }
}

/// Runtime library declarations for memory management and standard functions
pub mod runtime {
    use super::*;

    /// Generate runtime function declarations
    pub fn declare_runtime<'ctx>(codegen: &mut Codegen<'ctx>) {
        let ptr_ty = codegen.context.ptr_type(inkwell::AddressSpace::default());
        let i64_ty = codegen.context.i64_type();
        let void_ty = codegen.context.void_type();

        // Memory management
        // malloc(size: i64) -> *i8
        let malloc_ty = ptr_ty.fn_type(&[i64_ty.into()], false);
        codegen.module.add_function("malloc", malloc_ty, None);

        // free(ptr: *i8)
        let free_ty = void_ty.fn_type(&[ptr_ty.into()], false);
        codegen.module.add_function("free", free_ty, None);

        // calloc(count: i64, size: i64) -> *i8
        let calloc_ty = ptr_ty.fn_type(&[i64_ty.into(), i64_ty.into()], false);
        codegen.module.add_function("calloc", calloc_ty, None);

        // realloc(ptr: *i8, size: i64) -> *i8
        let realloc_ty = ptr_ty.fn_type(&[ptr_ty.into(), i64_ty.into()], false);
        codegen.module.add_function("realloc", realloc_ty, None);

        // I/O functions
        // printf(format: *const i8, ...) -> i32
        let i32_ty = codegen.context.i32_type();
        let printf_ty = i32_ty.fn_type(&[ptr_ty.into()], true); // varargs
        codegen.module.add_function("printf", printf_ty, None);

        // puts(s: *const i8) -> i32
        let puts_ty = i32_ty.fn_type(&[ptr_ty.into()], false);
        codegen.module.add_function("puts", puts_ty, None);
    }

    /// Helper to call malloc
    pub fn gen_malloc<'ctx>(
        codegen: &Codegen<'ctx>,
        size: inkwell::values::IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        let malloc_fn = codegen.module.get_function("malloc")
            .ok_or_else(|| CodegenError::UndefinedFunction("malloc".to_string()))?;

        let call_site = codegen.builder.build_call(malloc_fn, &[size.into()], "malloc_call")
            .map_err(|e| CodegenError::LlvmError(format!("malloc call failed: {:?}", e)))?;

        match call_site.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(BasicValueEnum::PointerValue(ptr)) => Ok(ptr),
            _ => Err(CodegenError::LlvmError("malloc did not return pointer".to_string())),
        }
    }

    /// Helper to call free
    pub fn gen_free<'ctx>(
        codegen: &Codegen<'ctx>,
        ptr: PointerValue<'ctx>,
    ) -> Result<(), CodegenError> {
        let free_fn = codegen.module.get_function("free")
            .ok_or_else(|| CodegenError::UndefinedFunction("free".to_string()))?;

        codegen.builder.build_call(free_fn, &[ptr.into()], "free_call")
            .map_err(|e| CodegenError::LlvmError(format!("free call failed: {:?}", e)))?;

        Ok(())
    }
}

/// Struct construction helpers
pub mod structs {
    use super::*;

    /// Build a struct value from field values
    pub fn build_struct<'ctx>(
        codegen: &Codegen<'ctx>,
        struct_ty: inkwell::types::StructType<'ctx>,
        field_values: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let mut struct_val = struct_ty.get_undef();

        for (i, &field_val) in field_values.iter().enumerate() {
            struct_val = codegen.builder.build_insert_value(struct_val, field_val, i as u32, "field")
                .map_err(|e| CodegenError::LlvmError(format!("insert_value failed: {:?}", e)))?
                .into_struct_value();
        }

        Ok(struct_val.into())
    }

    /// Extract a field from a struct
    pub fn extract_field<'ctx>(
        codegen: &Codegen<'ctx>,
        struct_val: inkwell::values::StructValue<'ctx>,
        field_index: u32,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        codegen.builder.build_extract_value(struct_val, field_index, "field_extract")
            .map_err(|e| CodegenError::LlvmError(format!("extract_value failed: {:?}", e)))
    }
}

/// Pattern matching codegen support
pub mod patterns {
    use super::*;

    /// Generate code for pattern matching
    /// Returns the basic block to jump to after the match
    pub fn gen_match<'ctx>(
        codegen: &mut Codegen<'ctx>,
        value: BasicValueEnum<'ctx>,
        arms: &[(i64, my_mir::BlockId)], // (pattern_value, target_block)
        default: my_mir::BlockId,
        blocks: &HashMap<my_mir::BlockId, inkwell::basic_block::BasicBlock<'ctx>>,
        fn_value: FunctionValue<'ctx>,
    ) -> Result<(), CodegenError> {
        if !value.is_int_value() {
            return Err(CodegenError::TypeError("pattern match value must be integer".to_string()));
        }

        let switch_val = value.into_int_value();
        let default_bb = blocks.get(&default)
            .ok_or_else(|| CodegenError::TypeError(format!("undefined block {:?}", default)))?;

        let llvm_cases: Vec<_> = arms.iter()
            .filter_map(|(val, block_id)| {
                blocks.get(block_id).map(|bb| {
                    (codegen.context.i64_type().const_int(*val as u64, false), *bb)
                })
            })
            .collect();

        codegen.builder.build_switch(switch_val, *default_bb, &llvm_cases)
            .map_err(|e| CodegenError::LlvmError(format!("build_switch failed: {:?}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_codegen() {
        let context = Context::create();
        let codegen = Codegen::new(&context, "test", TargetSpec::host());
        assert!(codegen.verify().is_ok());
    }
}
