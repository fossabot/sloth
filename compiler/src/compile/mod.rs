mod desugar;
mod error;
mod expression_compiler;
mod free_variable_finder;
mod module_compiler;
mod module_interface_compiler;
mod name_generator;
mod type_compiler;
mod type_inference;

use crate::ast;
use desugar::{desugar_with_types, desugar_without_types};
use error::CompileError;
use module_compiler::ModuleCompiler;
use module_interface_compiler::ModuleInterfaceCompiler;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use type_inference::infer_types;

pub fn compile(
    module_name: &str,
    module: &ast::Module,
    imported_modules: &[ast::ModuleInterface],
    destination: &str,
) -> Result<(), CompileError> {
    let module = desugar_with_types(&infer_types(&desugar_without_types(module))?);

    File::create(destination)?.write_all(core::compile::compile(&rename_top_level_variables(
        &ModuleCompiler::new().compile(&module, imported_modules)?,
        module_name,
    ))?)?;

    File::create(Path::new(destination).with_extension("json"))?.write_all(
        serde_json::to_string(&ModuleInterfaceCompiler::new().compile(&module))?.as_bytes(),
    )?;

    Ok(())
}

fn rename_top_level_variables(module: &core::ast::Module, module_name: &str) -> core::ast::Module {
    let mut names = HashMap::new();

    for definition in module.definitions() {
        names.insert(
            definition.name(),
            format!("{}.{}", module_name, definition.name()),
        );
    }

    names.insert("main", "sloth_main".into());

    core::ast::Module::new(
        module.declarations().iter().cloned().collect(),
        module
            .definitions()
            .iter()
            .map(|definition| match definition {
                core::ast::Definition::FunctionDefinition(function_definition) => {
                    core::ast::FunctionDefinition::new(
                        names
                            .get(function_definition.name())
                            .cloned()
                            .unwrap_or_else(|| function_definition.name().into()),
                        function_definition.environment().iter().cloned().collect(),
                        function_definition.arguments().iter().cloned().collect(),
                        function_definition.body().rename_variables(&names),
                        function_definition.result_type().clone(),
                    )
                    .into()
                }
                core::ast::Definition::ValueDefinition(value_definition) => {
                    core::ast::ValueDefinition::new(
                        names
                            .get(value_definition.name())
                            .cloned()
                            .unwrap_or_else(|| value_definition.name().into()),
                        value_definition.body().rename_variables(&names),
                        value_definition.type_().clone(),
                    )
                    .into()
                }
            })
            .collect(),
    )
}
