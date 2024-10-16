use crate::*;

pub fn cog_d(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  if cur.delims.is_none() {
    state.current().delims = Some(state.pool.get_string(vword.str_word.len()));
  }
  let delims = &mut state.current().delims;
  delims.as_mut().unwrap().clear();
  delims.as_mut().unwrap().push_str(&vword.str_word);

  state.pool.add_val(v);
  state
}

pub fn cog_i(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  if cur.ignored.is_none() {
    state.current().ignored = Some(state.pool.get_string(vword.str_word.len()));
  }
  let ignored = &mut state.current().ignored;
  ignored.as_mut().unwrap().clear();
  ignored.as_mut().unwrap().push_str(&vword.str_word);

  state.pool.add_val(v);
  state
}

pub fn cog_s(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  if cur.singlets.is_none() {
    state.current().singlets = Some(state.pool.get_string(vword.str_word.len()));
  }
  let singlets = &mut state.current().singlets;
  singlets.as_mut().unwrap().clear();
  singlets.as_mut().unwrap().push_str(&vword.str_word);

  state.pool.add_val(v);
  state
}

pub fn cog_dtgl(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state.current().dflag = !state.current_ref().dflag;
  state
}

pub fn cog_itgl(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state.current().iflag = !state.current_ref().iflag;
  state
}

pub fn cog_stgl(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state.current().sflag = !state.current_ref().sflag;
  state
}


pub fn cog_delim(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  let mut delims = cur.delims.take();
  for c in vword.str_word.chars() {
    if delims.is_none() {
      delims = Some(state.pool.get_string(DEFAULT_STRING_LENGTH));
    }
    if delims.as_ref().unwrap().chars().all(|x| x != c) {
      delims.as_mut().unwrap().push(c);
    }
  }
  state.current().delims = delims.take();
  state.pool.add_val(v);
  state
}

pub fn cog_ignore(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  let mut ignored = cur.ignored.take();
  for c in vword.str_word.chars() {
    if ignored.is_none() {
      ignored = Some(state.pool.get_string(DEFAULT_STRING_LENGTH));
    }
    if ignored.as_ref().unwrap().chars().all(|x| x != c) {
      ignored.as_mut().unwrap().push(c);
    }
  }
  state.current().ignored = ignored.take();
  state.pool.add_val(v);
  state
}

pub fn cog_singlet(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  let mut singlets = cur.singlets.take();
  for c in vword.str_word.chars() {
    if singlets.is_none() {
      singlets = Some(state.pool.get_string(DEFAULT_STRING_LENGTH));
    }
    if singlets.as_ref().unwrap().chars().all(|x| x != c) {
      singlets.as_mut().unwrap().push(c);
    }
  }
  state.current().singlets = singlets.take();
  state.pool.add_val(v);
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "d", cog_d);
  add_word!(state, "i", cog_i);
  add_word!(state, "s", cog_s);
  add_word!(state, "dtgl", cog_dtgl);
  add_word!(state, "itgl", cog_itgl);
  add_word!(state, "stgl", cog_stgl);
  add_word!(state, "delim", cog_delim);
  add_word!(state, "ignore", cog_ignore);
  add_word!(state, "singlet", cog_singlet);
}
