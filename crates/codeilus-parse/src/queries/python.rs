use super::LanguageQueries;

pub static QUERIES: &LanguageQueries = &LanguageQueries {
    definitions: r#"
        (function_definition name: (identifier) @name) @def
        (class_definition name: (identifier) @name) @def
    "#,
    imports: r#"
        (import_statement name: (dotted_name) @module) @import
        (import_from_statement
            module_name: (dotted_name) @module
            name: (dotted_name) @name) @import
        (import_from_statement
            module_name: (relative_import) @module
            name: (dotted_name) @name) @import
    "#,
    calls: r#"
        (call function: (identifier) @callee) @call
        (call function: (attribute attribute: (identifier) @callee)) @call
        (decorator (identifier) @callee) @call
    "#,
    heritage: r#"
        (class_definition
            name: (identifier) @child
            superclasses: (argument_list (identifier) @parent)) @heritage
    "#,
};
