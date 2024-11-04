pub mod combinators;
pub mod cranker;
pub mod errors;
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
  combinators::add_words(state);
  cranker::add_words(state);
  errors::add_words(state);
  io::add_words(state);
  math::add_words(state);
  metastack::add_words(state);
  misc::add_words(state);
  parser::add_words(state);
  stackops::add_words(state);
  strings::add_words(state);
  wordtable::add_words(state);
}
