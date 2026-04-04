#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod lang;
mod tr;

pub use lang::{Language, tr, tr_of};
pub use sys_locale::get_locale;
pub use tr::{en_US::en_US, jp::jp, zh_Hans::zh_Hans, zh_Hant::zh_Hant};
