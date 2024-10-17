pub mod cranker;
pub mod errors;
pub mod multithreading;
pub mod parser;
pub mod stackops;

use crate::CognitionState;
use crate::Value;

pub fn cog_nop(state: CognitionState, _v: Option<&Value>) -> CognitionState { state }

pub fn add_builtins(state: &mut CognitionState) {
  cranker::add_words(state);
  errors::add_words(state);
  parser::add_words(state);
  stackops::add_words(state);
  add_word!(state, "nop", cog_nop);
  add_word!(state, "nothing");
}
