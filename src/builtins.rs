pub mod combinators;
pub mod cranker;
pub mod errors;
pub mod fllibs;
pub mod io;
pub mod math;
pub mod metastack;
pub mod misc;
pub mod parser;
pub mod stackops;
pub mod strings;
pub mod wordtable;

use crate::CognitionState;

pub fn add_builtins(state: &mut CognitionState) {
  combinators::add_builtins(state);
  cranker::add_builtins(state);
  errors::add_builtins(state);
  fllibs::add_builtins(state);
  io::add_builtins(state);
  math::add_builtins(state);
  metastack::add_builtins(state);
  misc::add_builtins(state);
  parser::add_builtins(state);
  stackops::add_builtins(state);
  strings::add_builtins(state);
  wordtable::add_builtins(state);
}
