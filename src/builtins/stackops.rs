use crate::*;

pub fn cog_clear(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
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
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v1 = stack.pop().unwrap();
  let v2 = stack.pop().unwrap();
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

pub fn cog_ssize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let length = state.current().stack.len();
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  if length > isize::MAX as usize { return state.with_math(math).eval_error("OUT OF BOUNDS", w) }
  match math.math().itos(length as isize, &mut state) {
    Ok(s) => {
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state.set_math(math);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => { return state.with_math(math).eval_error(e, w) }
  }
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "clear", cog_clear);
  add_builtin!(state, "drop", cog_drop);
  add_builtin!(state, "swap", cog_swap);
  add_builtin!(state, "dup", cog_dup);
  add_builtin!(state, "ssize", cog_ssize);
}
