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
        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Bind parameters to locals
        for (i, param) in func.params.iter().enumerate() {
            let param_value = fn_value.get_nth_param(i as u32).unwrap();
            self.values.insert(param.id, param_value);
        }

        // Generate blocks
        // TODO: Implement full block generation from CFG

        // For now, just return void/unit
        match &func.return_type {
            MirType::Unit | MirType::Never => {
                self.builder.build_return(None).unwrap();
            }
            _ => {
                let zero = self.context.i64_type().const_int(0, false);
                self.builder.build_return(Some(&zero)).unwrap();
            }
        }

        Ok(())
    }

    /// Lower MIR type to LLVM type
    fn lower_type(&self, ty: &MirType) -> BasicTypeEnum<'ctx> {
        match ty {
            MirType::I32 => self.context.i32_type().into(),
            MirType::I64 => self.context.i64_type().into(),
            MirType::F32 => self.context.f32_type().into(),
            MirType::F64 => self.context.f64_type().into(),
            MirType::Bool => self.context.bool_type().into(),
            MirType::Ptr(inner) => self
                .lower_type(inner)
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
                .i8_type()
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

    /// Run optimization passes
    pub fn optimize(&self, level: OptLevel) {
        // TODO: Implement LLVM optimization passes
        // Using new pass manager API
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

/// AI runtime stub generator
pub mod ai_stubs {
    use super::*;

    /// Generate AI runtime stubs for linking
    pub fn generate_stubs<'ctx>(codegen: &mut Codegen<'ctx>) {
        // Declare external AI runtime functions
        let i8_ptr = codegen.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let i64_ty = codegen.context.i64_type();

        // my_ai_query(model: *const i8, prompt: *const i8) -> *const i8
        let query_ty = i8_ptr.fn_type(&[i8_ptr.into(), i8_ptr.into()], false);
        codegen.module.add_function("my_ai_query", query_ty, None);

        // my_ai_verify(condition: *const i8) -> i64 (bool)
        let verify_ty = i64_ty.fn_type(&[i8_ptr.into()], false);
        codegen.module.add_function("my_ai_verify", verify_ty, None);

        // my_ai_embed(text: *const i8) -> *const f32
        let f32_ptr = codegen.context.f32_type().ptr_type(inkwell::AddressSpace::default());
        let embed_ty = f32_ptr.fn_type(&[i8_ptr.into()], false);
        codegen.module.add_function("my_ai_embed", embed_ty, None);
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
