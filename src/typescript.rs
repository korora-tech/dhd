
use linkme;
use std::collections::HashMap;

// Distributed slices to collect TypeScript definitions at compile time
#[linkme::distributed_slice]
pub static TYPESCRIPT_FUNCTIONS: [(&'static str, &'static str)] = [..];

#[linkme::distributed_slice]
pub static TYPESCRIPT_TYPES: [(&'static str, &'static str)] = [..];

#[linkme::distributed_slice]
pub static TYPESCRIPT_METHODS: [(&'static str, &'static str)] = [..];

pub fn generate_typescript_definitions() -> String {
    let mut output = String::new();

    output.push_str("// Generated TypeScript definitions for DHD\n");
    output.push_str("// This file provides global type definitions for dhd modules\n\n");
    
    // Make all types and functions globally available
    output.push_str("declare global {\n");

    // Collect methods by type name
    let mut methods_map: HashMap<&str, &str> = HashMap::new();
    for (type_name, methods) in TYPESCRIPT_METHODS {
        methods_map.insert(type_name, methods);
    }

    // Add collected type definitions with their methods merged
    if !TYPESCRIPT_TYPES.is_empty() {
        output.push_str("  // Type definitions\n");
        for (type_name, definition) in TYPESCRIPT_TYPES {
            // Remove "export " from definitions since they're now global
            let global_definition = definition.replace("export ", "  ");
            
            // Check if we have methods for this type
            if let Some(methods) = methods_map.get(type_name) {
                // Parse the methods format: _METHODS_TypeName:{...}
                if let Some(methods_start) = methods.find('{') {
                    let methods_content = &methods[methods_start+1..methods.len()-1];
                    
                    // Merge the interface with its methods
                    if global_definition.contains("interface") && global_definition.ends_with("}") {
                        // Remove the closing brace and add methods
                        let interface_without_brace = &global_definition[..global_definition.len()-1];
                        output.push_str(&interface_without_brace);
                        if !interface_without_brace.trim().ends_with("{") {
                            output.push_str(";\n");
                        }
                        // Indent methods content
                        let indented_methods = methods_content.lines()
                            .map(|line| if line.trim().is_empty() { line.to_string() } else { format!("  {}", line) })
                            .collect::<Vec<_>>()
                            .join("\n");
                        output.push_str(&indented_methods);
                        output.push_str("\n  }");
                    } else {
                        // Just output the definition as-is if it's not an interface
                        output.push_str(&global_definition);
                    }
                }
            } else {
                // No methods for this type, output as-is
                output.push_str(&global_definition);
            }
            output.push_str("\n\n");
        }
    }

    // Add collected function signatures
    if !TYPESCRIPT_FUNCTIONS.is_empty() {
        output.push_str("  // Helper function signatures\n");
        for (_, signature) in TYPESCRIPT_FUNCTIONS {
            // Remove "export " and add proper indentation
            let global_signature = signature.replace("export ", "  ");
            output.push_str(&global_signature);
            output.push_str("\n");
        }
        output.push_str("\n");
    }

    // Close the declare global block
    output.push_str("}\n\n");
    
    // Add export statement to make this a module
    output.push_str("export {};\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_typescript_definitions() {
        // Since the distributed slices are populated at compile time,
        // we can only test that the function runs and produces output
        let output = generate_typescript_definitions();
        
        // Should at least have the header
        assert!(output.contains("// Generated TypeScript definitions for DHD"));
        
        // Should contain type definitions section if types exist
        if !TYPESCRIPT_TYPES.is_empty() {
            assert!(output.contains("// Type definitions"));
        }
        
        // Should contain function signatures section if functions exist
        if !TYPESCRIPT_FUNCTIONS.is_empty() {
            assert!(output.contains("// Helper function signatures"));
        }
    }

    #[test]
    fn test_methods_merging_logic() {
        // Test the method merging logic with sample data
        let mut methods_map = HashMap::new();
        methods_map.insert("TestType", "_METHODS_TestType:{    method1(): void;\n    method2(arg: string): number\n}");
        
        let interface_def = "export interface TestType {\n    field1: string\n}";
        
        // Simulate the merging logic
        if let Some(methods) = methods_map.get("TestType") {
            if let Some(methods_start) = methods.find('{') {
                let methods_content = &methods[methods_start+1..methods.len()-1];
                
                if interface_def.contains("export interface") && interface_def.ends_with("}") {
                    let interface_without_brace = &interface_def[..interface_def.len()-1];
                    let mut result = String::new();
                    result.push_str(interface_without_brace);
                    result.push_str(";\n");
                    result.push_str(methods_content);
                    result.push_str("\n}");
                    
                    assert!(result.contains("field1: string"));
                    assert!(result.contains("method1(): void"));
                    assert!(result.contains("method2(arg: string): number"));
                }
            }
        }
    }
}