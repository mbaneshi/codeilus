use super::LanguageQueries;

pub static QUERIES: &LanguageQueries = &LanguageQueries {
    definitions: r#"
        (function_item name: (identifier) @name) @def
        (struct_item name: (type_identifier) @name) @def
        (enum_item name: (type_identifier) @name) @def
        (trait_item name: (type_identifier) @name) @def
        (impl_item trait: (type_identifier) @trait_name type: (type_identifier) @name) @def
    "#,
    imports: r#"
        (use_declaration argument: (_) @module) @import
    "#,
    calls: r#"
        (call_expression function: (identifier) @callee) @call
        (call_expression function: (scoped_identifier name: (identifier) @callee)) @call
        (call_expression function: (field_expression field: (field_identifier) @callee)) @call
        (macro_invocation macro: (identifier) @callee) @call
    "#,
    heritage: r#"
        (impl_item trait: (type_identifier) @parent type: (type_identifier) @child) @heritage
    "#,
};
