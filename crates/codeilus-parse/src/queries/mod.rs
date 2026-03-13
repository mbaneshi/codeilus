pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust_lang;
pub mod typescript;

use codeilus_core::Language;

pub struct LanguageQueries {
    pub definitions: &'static str,
    pub imports: &'static str,
    pub calls: &'static str,
    pub heritage: &'static str,
}

pub fn get_queries(lang: Language) -> &'static LanguageQueries {
    match lang {
        Language::Python => python::QUERIES,
        Language::TypeScript => typescript::QUERIES,
        Language::JavaScript => javascript::QUERIES,
        Language::Rust => rust_lang::QUERIES,
        Language::Go => go::QUERIES,
        Language::Java => java::QUERIES,
        _ => python::QUERIES, // fallback; unsupported languages won't reach here
    }
}
