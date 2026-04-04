pub struct zh_Hans;

impl crate::Language for zh_Hans {
  fn hello_world(&self) -> String {
    "你好，AutoLang！".into()
  }
}
