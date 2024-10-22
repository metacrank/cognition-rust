use crate::*;

pub fn cog_crank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v) = cur.stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let stack = v.value_stack_ref();
  if stack.len() != 1 {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let word_val = &stack[0];
  let Value::Word(vword) = word_val else {
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(ref math) = cur.math else {
    return state.eval_error("MATH BASE ZERO", w)
  };
  if math.base() == 0 {
    return state.eval_error("MATH BASE ZERO", w)
  }
  let i = match math.stoi(&vword.str_word) {
    Ok(i) => if i > i32::MAX as isize {
      return state.eval_error("OUT OF BOUNDS", w)
    } else { i as i32 },
    Err(e) => return state.eval_error(e, w),
  };
  let v = cur.stack.pop().unwrap();
  state.pool.add_val(v);
  let cur = state.current();
  if cur.cranks.is_none() {
    state.current().cranks = Some(state.pool.get_cranks(1));
  }
  if let Some(crank) = state.current().cranks.as_mut().unwrap().first_mut() {
    crank.modulo = 0;
    crank.base = i;
  } else {
    state.current().cranks.as_mut().unwrap().push(Crank { modulo: 0, base: 1 });
  }
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "crank", cog_crank);
}
