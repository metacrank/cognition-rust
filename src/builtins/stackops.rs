use crate::*;

pub fn cog_clear(mut state: CognitionState, _v: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  while let Some(v) = cur.stack.pop() {
    state.pool.add_val(v);
  }
  state.push_cur(cur_v)
}

pub fn cog_drop(mut state: CognitionState, v: Option<&Value>) -> CognitionState {
  match state.current().stack.pop() {
    Some(v1) => { state.pool.add_val(v1); state },
    None     => { state.eval_error("", v) },
  }
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "clear", cog_clear);
  add_word!(state, "drop", cog_drop);
}
