pub struct ja;

impl crate::Language for ja {
  fn hello_world(&self) -> String {
    "初めまして、AutoLang！".into()
  }
}
