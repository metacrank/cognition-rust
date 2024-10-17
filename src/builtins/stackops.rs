use crate::*;

pub fn cog_clear(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  while let Some(v) = cur.stack.pop() {
    state.pool.add_val(v);
  }
  state.push_cur(cur_v)
}

pub fn cog_drop(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  match state.current().stack.pop() {
    Some(v1) => { state.pool.add_val(v1); state },
    None     => { state.eval_error("TOO FEW ARGUMEtTS", w) },
  }
}

pub fn cog_swap(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v1) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  stack.push(v1);
  stack.push(v2);
  state
}

pub fn cog_dup(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let new_v = state.value_copy(&v);
  state.current().stack.push(v);
  state.current().stack.push(new_v);
  state
}

// Does not use cognition math system
// Update when implemented
pub fn cog_ssize(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let size = state.current_ref().stack.len();
  let mut v = state.pool.get_vword(DEFAULT_STRING_LENGTH);
  let Value::Word(vword) = &mut v else { panic!("Pool::get_vword failed") };
  vword.str_word.push_str(&size.to_string());
  state.push_quoted(v);
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "clear", cog_clear);
  add_word!(state, "drop", cog_drop);
  add_word!(state, "swap", cog_swap);
  add_word!(state, "dup", cog_dup);
  add_word!(state, "ssize", cog_ssize);
}
