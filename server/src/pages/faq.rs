//! Logic for the faq page.

use langpack::{self, Locale};

/// The context object passed to `templates/faq.html.tera`
#[derive(Serialize)]
pub struct Context<'a> {
    pub page_title: &'a str,
    pub language: langpack::Language,
    pub strings: &'a langpack::Translations,
}

impl<'a> Context<'a> {
    pub fn new(locale: &'a Locale) -> Context<'a> {
        Context {
            page_title: &locale.strings.header.faq,
            strings: locale.strings,
            language: locale.language,
        }
    }
}
