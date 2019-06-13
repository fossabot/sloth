use super::super::ast;
use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::llvm;
use super::type_compiler::TypeCompiler;
use std::collections::HashMap;

const GLOBAL_INITIALIZER_NAME: &str = "sloth_init";

pub struct ModuleCompiler<'a> {
    module: &'a llvm::Module,
    ast_module: &'a ast::Module,
    type_compiler: TypeCompiler,
    variables: HashMap<String, llvm::Value>,
    initializers: Vec<llvm::Value>,
}

impl<'a> ModuleCompiler<'a> {
    pub fn new(module: &'a llvm::Module, ast_module: &'a ast::Module) -> ModuleCompiler<'a> {
        ModuleCompiler {
            module,
            ast_module,
            type_compiler: TypeCompiler::new(),
            variables: HashMap::new(),
            initializers: vec![],
        }
    }

    pub fn compile(&mut self) -> Result<(), CompileError> {
        unsafe {
            self.module.declare_intrinsics();

            for definition in self.ast_module.definitions() {
                match definition {
                    ast::Definition::FunctionDefinition(function_definition) => {
                        self.declare_function(function_definition)
                    }
                    ast::Definition::VariableDefinition(variable_definition) => {
                        self.declare_global_variable(variable_definition)
                    }
                }
            }

            for definition in self.ast_module.definitions() {
                match definition {
                    ast::Definition::FunctionDefinition(function_definition) => {
                        self.compile_function(function_definition)?
                    }
                    ast::Definition::VariableDefinition(variable_definition) => {
                        self.compile_global_variable(variable_definition)?
                    }
                }
            }

            self.compile_global_initializer();

            llvm::verify_module(&self.module);
        }

        Ok(())
    }

    unsafe fn declare_function(&mut self, function_definition: &ast::FunctionDefinition) {
        self.variables.insert(
            function_definition.name().into(),
            self.module.add_global(
                function_definition.name(),
                llvm::Type::struct_(&[llvm::Type::pointer(
                    self.type_compiler
                        .compile_function(function_definition.type_()),
                )]),
            ),
        );
    }

    unsafe fn compile_function(
        &self,
        function_definition: &ast::FunctionDefinition,
    ) -> Result<(), CompileError> {
        let closure = self.variables[function_definition.name()];
        let entry_function = self.module.add_function(
            &Self::generate_closure_entry_name(function_definition.name()),
            self.type_compiler
                .compile_function(&function_definition.type_()),
        );
        closure.set_initializer(llvm::const_struct(&[entry_function]));

        let mut arguments = self.variables.clone();

        for (index, name) in function_definition.arguments().iter().enumerate() {
            arguments.insert(
                name.clone(),
                llvm::get_param(entry_function, index as u32 + 1),
            );
        }

        let builder = llvm::Builder::new(entry_function);
        builder.position_at_end(builder.append_basic_block("entry"));
        builder.build_ret(
            ExpressionCompiler::new(&builder, &arguments).compile(&function_definition.body())?,
        );

        llvm::verify_function(entry_function);

        Ok(())
    }

    unsafe fn declare_global_variable(&mut self, variable_definition: &ast::VariableDefinition) {
        self.variables.insert(
            variable_definition.name().into(),
            self.module.add_global(
                variable_definition.name(),
                self.type_compiler.compile(variable_definition.type_()),
            ),
        );
    }

    unsafe fn compile_global_variable(
        &mut self,
        variable_definition: &ast::VariableDefinition,
    ) -> Result<(), CompileError> {
        let global = self.variables[variable_definition.name()];
        global.set_initializer(llvm::get_undef(global.type_().element()));

        let initializer = self.module.add_function(
            &Self::generate_initializer_name(variable_definition.name()),
            llvm::Type::function(llvm::Type::void(), &[]),
        );

        let builder = llvm::Builder::new(initializer);
        builder.position_at_end(builder.append_basic_block("entry"));
        builder.build_store(
            ExpressionCompiler::new(&builder, &self.variables)
                .compile(&variable_definition.body())?,
            global,
        );
        builder.build_ret_void();

        llvm::verify_function(initializer);
        self.initializers.push(initializer);

        Ok(())
    }

    unsafe fn compile_global_initializer(&self) {
        let initializer = self.module.add_function(
            GLOBAL_INITIALIZER_NAME,
            llvm::Type::function(llvm::Type::void(), &[]),
        );
        let builder = llvm::Builder::new(initializer);
        builder.position_at_end(builder.append_basic_block("entry"));

        for initializer in &self.initializers {
            builder.build_call(*initializer, &[]);
        }

        builder.build_ret_void();
    }

    fn generate_initializer_name(name: &str) -> String {
        [name, ".$init"].concat()
    }

    fn generate_closure_entry_name(name: &str) -> String {
        [name, ".$entry"].concat()
    }
}
