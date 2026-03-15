use super::LanguageQueries;

pub static QUERIES: &LanguageQueries = &LanguageQueries {
    definitions: r#"
        (class_declaration name: (identifier) @name) @def
        (method_declaration name: (identifier) @name) @def
        (interface_declaration name: (identifier) @name) @def
    "#,
    imports: r#"
        (import_declaration (scoped_identifier) @module) @import
    "#,
    calls: r#"
        (method_invocation name: (identifier) @callee) @call
        (object_creation_expression type: (type_identifier) @callee) @call
    "#,
    heritage: r#"
        (class_declaration
            name: (identifier) @child
            (superclass (type_identifier) @parent)) @heritage
        (class_declaration
            name: (identifier) @child
            (super_interfaces (type_list (type_identifier) @parent))) @heritage
    "#,
};
