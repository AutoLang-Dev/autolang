pub struct jp;

impl crate::Language for jp {
  fn hello_world(&self) -> String {
    "初めまして、AutoLang！".into()
  }
}
