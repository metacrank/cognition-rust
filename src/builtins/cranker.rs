use crate::*;

pub fn cog_crank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  // minimal crank, only handles crank 1
  let cur = state.current();
  let Some(v) = cur.stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let stack = v.value_stack_ref();
  if stack.len() != 1 {
    cur.stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let word_val = &stack[0];
  let Value::Word(vword) = word_val else {
    cur.stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };

  if vword.str_word.as_str() != "1" {
    return state.eval_error("BAD ARGUMENT (not 1)", w);
  }

  if cur.cranks.is_none() {
    state.current().cranks = Some(state.pool.get_cranks(1));
  }
  if let Some(crank) = state.current().cranks.as_mut().unwrap().first_mut() {
    crank.modulo = 0;
    crank.base = 1;
  } else {
    state.current().cranks.as_mut().unwrap().push(Crank { modulo: 0, base: 1 });
  }

  state.pool.add_val(v);
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "crank", cog_crank);
}
