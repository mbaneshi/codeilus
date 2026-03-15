use super::LanguageQueries;

pub static QUERIES: &LanguageQueries = &LanguageQueries {
    definitions: r#"
        (function_declaration name: (identifier) @name) @def
        (class_declaration name: (type_identifier) @name) @def
        (interface_declaration name: (type_identifier) @name) @def
        (method_definition name: (property_identifier) @name) @def
    "#,
    imports: r#"
        (import_statement
            source: (string) @module) @import
    "#,
    calls: r#"
        (call_expression function: (identifier) @callee) @call
        (call_expression function: (member_expression property: (property_identifier) @callee)) @call
        (new_expression constructor: (identifier) @callee) @call
    "#,
    heritage: r#"
        (class_declaration
            name: (type_identifier) @child
            (class_heritage
                (extends_clause value: (identifier) @parent))) @heritage
        (class_declaration
            name: (type_identifier) @child
            (class_heritage
                (implements_clause (type_identifier) @parent))) @heritage
    "#,
};
