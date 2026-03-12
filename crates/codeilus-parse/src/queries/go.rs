use super::LanguageQueries;

pub static QUERIES: &LanguageQueries = &LanguageQueries {
    definitions: r#"
        (function_declaration name: (identifier) @name) @def
        (method_declaration name: (field_identifier) @name) @def
        (type_declaration (type_spec name: (type_identifier) @name)) @def
    "#,
    imports: r#"
        (import_declaration (import_spec path: (interpreted_string_literal) @module)) @import
        (import_declaration (import_spec_list (import_spec path: (interpreted_string_literal) @module))) @import
    "#,
    calls: r#"
        (call_expression function: (identifier) @callee) @call
        (call_expression function: (selector_expression field: (field_identifier) @callee)) @call
    "#,
    heritage: "",
};
