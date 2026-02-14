use super::action_ref_to_interface_name;
use crate::fetcher::{ActionInput, ActionMetadata};

pub fn generate_type_definition(action_ref: &str, metadata: &ActionMetadata) -> String {
    let interface_name = action_ref_to_interface_name(action_ref);

    let mut output = String::new();

    // Header comment
    output.push_str(&format!(
        "// Auto-generated from {}\n// Do not edit manually\n\n",
        action_ref
    ));

    // Import JobStep from base
    output.push_str("import type { JobStep } from './base';\n\n");

    // Generate interface for inputs
    let inputs_interface = generate_inputs_interface(&interface_name, metadata);
    output.push_str(&inputs_interface);
    output.push_str("\n\n");

    // Generate outputs interface if present
    if let Some(outputs) = &metadata.outputs {
        if !outputs.is_empty() {
            let outputs_interface = generate_outputs_interface(&interface_name, outputs);
            output.push_str(&outputs_interface);
            output.push_str("\n\n");
        }
    }

    output
}

fn generate_inputs_interface(interface_name: &str, metadata: &ActionMetadata) -> String {
    let mut output = String::new();

    // JSDoc for the interface
    output.push_str(&format!(
        "/**\n * {}\n",
        metadata.description.as_deref().unwrap_or(&metadata.name)
    ));
    output.push_str(&format!(
        " * @see https://github.com/{}\n */\n",
        interface_name.to_lowercase().replace("v", "/v")
    ));

    output.push_str(&format!("export interface {}Inputs {{\n", interface_name));

    if let Some(inputs) = &metadata.inputs {
        for (name, input) in inputs {
            output.push_str(&generate_input_field(name, input));
        }
    }

    output.push('}');

    output
}

fn generate_input_field(name: &str, input: &ActionInput) -> String {
    let mut output = String::new();

    // JSDoc for the field
    output.push_str("    /**\n");
    if let Some(desc) = &input.description {
        for line in desc.lines() {
            output.push_str(&format!("     * {}\n", line.trim()));
        }
    }
    if let Some(default) = &input.default {
        output.push_str(&format!("     * @default {}\n", default));
    }
    if let Some(deprecation) = &input.deprecation_message {
        output.push_str(&format!("     * @deprecated {}\n", deprecation));
    }
    output.push_str("     */\n");

    // Field declaration
    let is_required = input.required.unwrap_or(false);
    let optional_marker = if is_required { "" } else { "?" };
    let field_type = infer_type_from_input(input);

    // Handle field names with special characters
    let field_name = if name.contains('-') || name.contains('.') {
        format!("'{}'", name)
    } else {
        name.to_string()
    };

    output.push_str(&format!(
        "    {}{}: {};\n",
        field_name, optional_marker, field_type
    ));

    output
}

fn generate_outputs_interface(
    interface_name: &str,
    outputs: &std::collections::HashMap<String, crate::fetcher::ActionOutput>,
) -> String {
    let mut output = String::new();

    output.push_str(&format!("export interface {}Outputs {{\n", interface_name));

    for (name, action_output) in outputs {
        // JSDoc
        if let Some(desc) = &action_output.description {
            output.push_str("    /**\n");
            for line in desc.lines() {
                output.push_str(&format!("     * {}\n", line.trim()));
            }
            output.push_str("     */\n");
        }

        // Handle field names with special characters
        let field_name = if name.contains('-') || name.contains('.') {
            format!("'{}'", name)
        } else {
            name.to_string()
        };

        output.push_str(&format!("    {}: string;\n", field_name));
    }

    output.push('}');

    output
}

fn infer_type_from_input(input: &ActionInput) -> &'static str {
    // Try to infer type from default value
    if let Some(default) = &input.default {
        let default_lower = default.to_lowercase();

        // Boolean detection
        if default_lower == "true" || default_lower == "false" {
            return "boolean";
        }

        // Number detection (simple check)
        if default.parse::<i64>().is_ok() || default.parse::<f64>().is_ok() {
            // But if it looks like a version, keep as string
            if !default.contains('.') || default.parse::<f64>().is_ok() {
                return "number";
            }
        }
    }

    // Default to string
    "string"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_type_definition() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "repository".to_string(),
            ActionInput {
                description: Some("Repository name".to_string()),
                required: Some(false),
                default: Some("${{ github.repository }}".to_string()),
                deprecation_message: None,
            },
        );

        let metadata = ActionMetadata {
            name: "Checkout".to_string(),
            description: Some("Checkout a Git repository".to_string()),
            inputs: Some(inputs),
            outputs: None,
            runs: None,
        };

        let result = generate_type_definition("actions/checkout@v5", &metadata);

        assert!(result.contains("ActionsCheckoutV5Inputs"));
        assert!(result.contains("repository?:"));
        assert!(result.contains("@default"));
    }

    #[test]
    fn test_infer_boolean_type() {
        let input = ActionInput {
            description: None,
            required: None,
            default: Some("true".to_string()),
            deprecation_message: None,
        };
        assert_eq!(infer_type_from_input(&input), "boolean");
    }

    #[test]
    fn test_infer_number_type() {
        let input = ActionInput {
            description: None,
            required: None,
            default: Some("42".to_string()),
            deprecation_message: None,
        };
        assert_eq!(infer_type_from_input(&input), "number");
    }
}
