use wasm3::WasmType;

use crate::{
    ast::Node,
    error::{Result, SWLError},
    utils,
};

pub trait WasmTypeName {
    fn wasm_type_name() -> &'static str;
}

impl WasmTypeName for i32 {
    fn wasm_type_name() -> &'static str {
        "i32"
    }
}

impl WasmTypeName for i64 {
    fn wasm_type_name() -> &'static str {
        "i64"
    }
}

impl WasmTypeName for f32 {
    fn wasm_type_name() -> &'static str {
        "f32"
    }
}

impl WasmTypeName for f64 {
    fn wasm_type_name() -> &'static str {
        "f64"
    }
}

pub fn eval_expr<V: WasmType + WasmTypeName>(node: &Node, prelude: &str) -> Result<V> {
    let expr = node
        .items
        .get(0)
        .ok_or(SWLError::Simple("Constexpr is missing expression".into()))?;

    let typ = V::wasm_type_name();

    let wat = format!(
        r#"
					(module
							{prelude}
							(func (export "main") (result {typ})
									{expr}
							)
					)
			"#
    );

    utils::run_wat::<V>(&wat)
}
