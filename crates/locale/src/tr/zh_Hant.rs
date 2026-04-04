pub struct zh_Hant;

impl crate::Language for zh_Hant {
  fn hello_world(&self) -> String {
    "你好，AutoLang！".into()
  }
}
