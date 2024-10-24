use crate::*;

pub fn cog_compose(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = stack.last_mut() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  v2.value_stack().reverse(); // TODO: avoid this reversion
  while let Some(v) = v2.value_stack().pop() {
    v1.value_stack().push(v);
  }
  state.pool.add_val(v2);
  state
}

pub fn add_words(state: &mut CognitionState) {
  let nopy = builtins::cog_nop;
  add_word!(state, "eval", nopy, EVAL);
  add_word!(state, "compose", cog_compose);
}
