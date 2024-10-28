use crate::*;

pub fn cog_cd(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if !v.is_stack() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  state.stack.push(v);
  state
}

pub fn cog_uncd(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let child = state.stack.pop().expect("Cognition metastack was empty");
  if state.stack.len() == 0 {
    let mut new_stack = state.pool.get_vstack(DEFAULT_STACK_SIZE);
    new_stack.container.faliases = state.default_faliases();
    state.stack.push(Value::Stack(new_stack));
  }
  state.current().stack.push(child);
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "cd", cog_cd);
  add_word!(state, "uncd", cog_uncd);
}
