use std::sync::RwLock;
use unic_langid::{LanguageIdentifier, lang, langid, script};

use super::tr::*;

pub trait Language: Sync {
  fn hello_world(&self) -> String {
    "Hello AutoLang!".into()
  }
}

pub fn tr_of(locale: String) -> &'static dyn Language {
  let LanguageIdentifier {
    language: lang,
    script,
    ..
  } = locale.parse().unwrap_or(langid!("en-US"));

  if lang == lang!("en") {
    &en_US::en_US
  } else if lang == lang!("jp") {
    &jp::jp
  } else if lang == lang!("zh") {
    if script == Some(script!("Hant")) {
      &zh_Hant::zh_Hant
    } else {
      &zh_Hans::zh_Hans
    }
  } else {
    &en_US::en_US
  }
}

static TR: RwLock<Option<&'static dyn Language>> = RwLock::new(None);

pub fn set_tr(locale: String) {
  *TR.write().unwrap() = Some(tr_of(locale));
}

pub fn tr() -> &'static dyn Language {
  if let Some(lang) = *TR.read().unwrap() {
    return lang;
  }

  let locale = std::env::var("AUTOLANG_LOCALE")
    .ok()
    .filter(|locale| !locale.trim().is_empty())
    .or_else(crate::get_locale)
    .unwrap_or_else(|| "en-US".into());

  set_tr(locale);
  TR.read().unwrap().unwrap()
}
