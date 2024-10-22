macro_rules! get_char {
  ($state:ident,$c:pat,$w:ident) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let Some($c) = iter.next() else { return $state.eval_error("BAD ARGUMENT TYPE", $w) };
    if iter.next().is_some() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
}

pub mod combinators;
pub mod cranker;
pub mod errors;
pub mod math;
pub mod multithreading;
pub mod parser;
pub mod stackops;

use crate::CognitionState;
use crate::Value;

pub fn cog_nop(state: CognitionState, _v: Option<&Value>) -> CognitionState { state }

pub fn add_builtins(state: &mut CognitionState) {
  combinators::add_words(state);
  cranker::add_words(state);
  errors::add_words(state);
  math::add_words(state);
  multithreading::add_words(state);
  parser::add_words(state);
  stackops::add_words(state);
  add_word!(state, "nop", cog_nop);
  add_word!(state, "nothing");
}
