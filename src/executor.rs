use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use rquickjs::{function::Func, Context as JsContext, Runtime as JsRuntime};

/// Output from a single __gha_build call
#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub id: String,
    pub json: String,
    /// "workflow" or "action"
    pub output_type: String,
}

/// Strip TypeScript types from source code, producing plain JavaScript.
/// Uses the oxc pipeline: Parser -> SemanticBuilder -> Transformer -> Codegen
pub fn strip_typescript(source: &str, filename: &str) -> Result<String> {
    let allocator = Allocator::default();
    let source_type =
        SourceType::from_path(Path::new(filename)).unwrap_or_else(|_| SourceType::tsx());

    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    if !parser_ret.errors.is_empty() {
        let errors: Vec<String> = parser_ret.errors.iter().map(|e| e.to_string()).collect();
        return Err(anyhow::anyhow!("Parse errors:\n{}", errors.join("\n")));
    }

    let mut program = parser_ret.program;

    let semantic_ret = SemanticBuilder::new().build(&program);
    let scoping = semantic_ret.semantic.into_scoping();

    let transform_options = TransformOptions::default();
    let _transformer_ret = Transformer::new(&allocator, Path::new(filename), &transform_options)
        .build_with_scoping(scoping, &mut program);

    let code = Codegen::new().build(&program).code;
    Ok(code)
}

/// Bundle runtime JS and workflow TS, then execute with QuickJS.
/// Returns a list of build outputs (workflow/action JSON).
pub fn execute_workflow(workflow_path: &Path, runtime_js_path: &Path) -> Result<Vec<BuildOutput>> {
    // Read the workflow TypeScript source
    let workflow_source = std::fs::read_to_string(workflow_path)
        .with_context(|| format!("Failed to read workflow file: {}", workflow_path.display()))?;

    // Read the runtime JS
    let runtime_js = std::fs::read_to_string(runtime_js_path)
        .with_context(|| format!("Failed to read runtime JS: {}", runtime_js_path.display()))?;

    // Strip TypeScript types from the workflow
    let filename = workflow_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let workflow_js = strip_typescript(&workflow_source, &filename)?;

    // Remove import/export statements from both sources for QuickJS script mode
    // (QuickJS eval runs in script mode, not ES module mode)
    let runtime_js = remove_imports(&runtime_js);
    let workflow_js = remove_imports(&workflow_js);

    // Bundle: runtime first, then workflow code
    let bundled = format!("{}\n\n{}", runtime_js, workflow_js);

    // Execute with QuickJS
    execute_js(&bundled)
}

/// Remove import/export statements from JavaScript source.
/// This is needed because we inline the runtime code.
pub fn remove_imports(source: &str) -> String {
    let mut result = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        // Skip import statements
        if trimmed.starts_with("import ") || trimmed.starts_with("import{") {
            continue;
        }
        // Skip export statements but keep the content
        if trimmed.starts_with("export ") {
            // "export const x = ..." -> "const x = ..."
            // "export function f()" -> "function f()"
            // "export default" -> skip
            // "export {" -> skip
            // "export type {" -> skip
            if trimmed.starts_with("export default ") {
                result.push(trimmed.trim_start_matches("export default ").to_string());
            } else if trimmed.starts_with("export {") || trimmed.starts_with("export type ") {
                continue;
            } else {
                result.push(trimmed.replacen("export ", "", 1));
            }
            continue;
        }
        result.push(line.to_string());
    }
    result.join("\n")
}

/// Register __gha_build host function and evaluate JavaScript with QuickJS.
/// Uses Rc/RefCell pattern to capture build outputs from JS callbacks.
pub fn execute_js(code: &str) -> Result<Vec<BuildOutput>> {
    let outputs: Rc<RefCell<Vec<BuildOutput>>> = Rc::new(RefCell::new(Vec::new()));

    {
        let rt = JsRuntime::new().context("Failed to create QuickJS runtime")?;
        let ctx = JsContext::full(&rt).context("Failed to create QuickJS context")?;

        let code_owned = code.to_string();

        ctx.with(|ctx| {
            let outputs_clone = outputs.clone();

            // Register __gha_build(id, json, type) host function
            let build_fn = Func::from(
                move |id: String, json: String, output_type: rquickjs::function::Opt<String>| {
                    outputs_clone.borrow_mut().push(BuildOutput {
                        id,
                        json,
                        output_type: output_type.0.unwrap_or_else(|| "workflow".to_string()),
                    });
                },
            );

            ctx.globals()
                .set("__gha_build", build_fn)
                .map_err(|e| anyhow::anyhow!("Failed to set __gha_build: {}", e))?;

            // Evaluate the bundled JavaScript
            ctx.eval::<(), _>(code_owned.as_bytes())
                .map_err(|e| anyhow::anyhow!("QuickJS evaluation error: {}", e))?;

            Ok::<_, anyhow::Error>(())
        })?;

        // ctx and rt are dropped here, releasing the Rc clone held by the Func
    }

    // Extract the outputs - Rc::try_unwrap succeeds because ctx/rt are dropped
    let result = Rc::try_unwrap(outputs)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap Rc - references still held"))?
        .into_inner();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_typescript_basic() {
        let ts_source = "const x: number = 42;\nconst y: string = \"hello\";";
        let result = strip_typescript(ts_source, "test.ts").unwrap();
        assert!(result.contains("const x = 42"));
        assert!(result.contains("const y = \"hello\""));
        assert!(!result.contains(": number"));
        assert!(!result.contains(": string"));
    }

    #[test]
    fn test_remove_imports() {
        let source = r#"import { getAction } from "./index";
import type { Job } from "./base";
const x = 1;
export const y = 2;
export { z };
export type { Foo };
"#;
        let result = remove_imports(source);
        assert!(!result.contains("import"));
        assert!(result.contains("const x = 1"));
        assert!(result.contains("const y = 2"));
        assert!(!result.contains("export {"));
        assert!(!result.contains("export type"));
    }

    #[test]
    fn test_remove_imports_strips_export_class() {
        let source = "export class Foo {}\nexport function bar() {}";
        let result = remove_imports(source);
        assert!(result.contains("class Foo {}"));
        assert!(result.contains("function bar() {}"));
        assert!(!result.contains("export"));
    }

    #[test]
    fn test_execute_js_basic() {
        let code = r#"
            function __test() {
                __gha_build("test-workflow", '{"name":"test","on":{"push":{}},"jobs":{}}', "workflow");
            }
            __test();
        "#;
        let outputs = execute_js(code).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].id, "test-workflow");
        assert_eq!(outputs[0].output_type, "workflow");
    }

    #[test]
    fn test_execute_js_multiple_outputs() {
        let code = r#"
            __gha_build("wf1", '{"name":"first"}', "workflow");
            __gha_build("wf2", '{"name":"second"}', "workflow");
            __gha_build("act1", '{"name":"action1"}', "action");
        "#;
        let outputs = execute_js(code).unwrap();
        assert_eq!(outputs.len(), 3);
        assert_eq!(outputs[0].output_type, "workflow");
        assert_eq!(outputs[2].output_type, "action");
    }

    /// End-to-end test: runtime JS + Job/Workflow classes → QuickJS → JSON output
    #[test]
    fn test_job_workflow_pipeline() {
        use crate::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE;

        // Simulate what generate_index_js produces
        let runtime_js = format!(
            r#"export function getAction(ref) {{
    return function(config) {{
        if (config === undefined) config = {{}};
        var step = {{ uses: ref }};
        if (config.name !== undefined) step.name = config.name;
        if (config.with !== undefined) step.with = config.with;
        return step;
    }};
}}
{}"#,
            JOB_WORKFLOW_RUNTIME_TEMPLATE
        );

        // Simulate a workflow TS after type-stripping
        let workflow_js = r#"
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");

new Workflow({
    name: "CI",
    on: { push: { branches: ["main"] } },
}).jobs(j => j
    .add("build",
        new Job("ubuntu-latest")
            .steps(s => s
                .add(checkout({ name: "Checkout", with: { "fetch-depth": 1 } }))
                .add({ name: "Test", run: "npm test" })
            )
    )
).build("ci");
"#;

        // Bundle like execute_workflow does
        let runtime_stripped = remove_imports(&runtime_js);
        let workflow_stripped = remove_imports(workflow_js);
        let bundled = format!("{}\n\n{}", runtime_stripped, workflow_stripped);

        let outputs = execute_js(&bundled).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].id, "ci");
        assert_eq!(outputs[0].output_type, "workflow");

        // Verify the JSON structure
        let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
        assert_eq!(json["name"], "CI");
        assert!(json["on"]["push"]["branches"].is_array());
        assert_eq!(json["jobs"]["build"]["runs-on"], "ubuntu-latest");

        let steps = json["jobs"]["build"]["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0]["uses"], "actions/checkout@v5");
        assert_eq!(steps[0]["name"], "Checkout");
        assert_eq!(steps[1]["run"], "npm test");
    }

    /// Test Action (formerly CompositeAction) through QuickJS
    #[test]
    fn test_composite_action_pipeline() {
        use crate::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE;

        let runtime_js = format!(
            "function getAction(ref) {{ return function(config) {{ return {{ uses: ref }}; }}; }}\n{}",
            JOB_WORKFLOW_RUNTIME_TEMPLATE
        );

        let workflow_js = r#"
new Action({
    name: "My Action",
    description: "A composite action",
})
    .steps(s => s
        .add({ name: "Step 1", run: "echo hello", shell: "bash" })
    )
    .build("my-action");
"#;

        let runtime_stripped = remove_imports(&runtime_js);
        let bundled = format!("{}\n\n{}", runtime_stripped, workflow_js);

        let outputs = execute_js(&bundled).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].id, "my-action");
        assert_eq!(outputs[0].output_type, "action");

        let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
        assert_eq!(json["name"], "My Action");
        assert_eq!(json["runs"]["using"], "composite");
        assert_eq!(json["runs"]["steps"][0]["run"], "echo hello");
    }

    /// Test full TS→JS strip + QuickJS execution
    #[test]
    fn test_strip_then_execute() {
        use crate::generator::templates::JOB_WORKFLOW_RUNTIME_TEMPLATE;

        let runtime_js = format!(
            "function getAction(ref) {{ return function(config) {{ return {{ uses: ref }}; }}; }}\n{}",
            JOB_WORKFLOW_RUNTIME_TEMPLATE
        );

        // TypeScript source with type annotations
        let ts_source = r#"
import { Job, Workflow } from "../../generated/index.js";

const wf: Workflow = new Workflow({
    name: "Typed",
    on: { push: {} },
}).jobs(j => j
    .add("job1",
        new Job("ubuntu-latest")
            .steps(s => s
                .add({ name: "Hello", run: "echo hi" })
            )
    )
);

wf.build("typed-wf");
"#;

        // Strip TS types
        let js = strip_typescript(ts_source, "test.ts").unwrap();

        // Bundle
        let runtime_stripped = remove_imports(&runtime_js);
        let workflow_stripped = remove_imports(&js);
        let bundled = format!("{}\n\n{}", runtime_stripped, workflow_stripped);

        let outputs = execute_js(&bundled).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].id, "typed-wf");

        let json: serde_json::Value = serde_json::from_str(&outputs[0].json).unwrap();
        assert_eq!(json["name"], "Typed");
        assert_eq!(json["jobs"]["job1"]["steps"][0]["run"], "echo hi");
    }
}
