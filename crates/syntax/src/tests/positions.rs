//! Accessors that pick a child by position, checked on the shapes they disambiguate.

use crate::Red;
use crate::ast;
use crate::ast::Node;
use crate::frag;

#[test]
fn fn_type_ret_is_the_return_type() {
  let (green, src) = frag! {
    FnType {
      TupleType {
        OpenParen
        TypeField {
          Name { Ident("next") }
          Colon
          PathType { Path { PathSegment { Name { Ident("Int") } } } }
        }
        CloseParen
      }
      ThinArrow
      PathType { Path { PathSegment { Name { Ident("Out") } } } }
    }
  }
  .finish();

  let ty = ast::FnType::new(Red::new_root(green)).expect("FnType");

  assert_eq!(&src[ty.params().red().range()], "(next:Int)");
  assert_eq!(&src[ty.ret().red().range()], "Out");
}
