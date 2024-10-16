pub mod multithreading;
pub mod stackops;

use crate::CognitionState;
use crate::Value;

pub fn cog_nop(state: CognitionState, _v: Option<&Value>) -> CognitionState { state }

pub fn add_builtins(mut state: CognitionState) -> CognitionState {
  stackops::add_words(&mut state);
  add_word!(state, "nop", cog_nop);
  state
}