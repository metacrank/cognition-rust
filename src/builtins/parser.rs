use crate::*;

pub fn cog_getf(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let faliases = state.current().faliases.take();
  if faliases.is_none() {
    let list = state.pool.get_vstack(DEFAULT_STACK_SIZE);
    state.current().stack.push(Value::Stack(list));
    return state;
  }
  let faliases = faliases.unwrap();
  let mut list = state.pool.get_vstack(faliases.len());
  for alias in faliases.iter() {
    let mut v = state.pool.get_vword(alias.len());
    v.str_word.push_str(&alias);
    list.container.stack.push(Value::Word(v));
  }
  state.current().faliases = Some(faliases);
  state.current().stack.push(Value::Stack(list));
  state
}

pub fn cog_setf(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(list) = state.current().stack.pop() else {
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let lstack = list.value_stack_ref();
  if lstack.len() == 0 {
    if let Some(faliases) = state.current().faliases.take() {
      state.pool.add_faliases(faliases);
    }
    state.pool.add_val(list);
    return state;
  }
  if lstack.iter().any(|x| !x.is_word()) {
    state.current().stack.push(list);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  if state.current().faliases.is_none() {
    state.current().faliases = Some(state.pool.get_faliases());
  }
  let mut faliases = state.current().faliases.take().unwrap();
  for s in faliases.drain() { state.pool.add_string(s); }
  for s in lstack.iter() { faliases.insert(state.string_copy(&s.vword_ref().str_word)); }
  state.current().faliases = Some(faliases);
  state.pool.add_val(list);
  state
}

pub fn cog_aliasf(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(list) = state.current().stack.pop() else {
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let lstack = list.value_stack_ref();
  if lstack.len() == 0 {
    state.pool.add_val(list);
    return state;
  }
  if lstack.iter().any(|x| !x.is_word()) {
    state.current().stack.push(list);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  if state.current().faliases.is_none() {
    state.current().faliases = Some(state.pool.get_faliases());
  }
  let mut faliases = state.current().faliases.take().unwrap();
  for v in lstack.iter() { faliases.insert(state.string_copy(&v.vword_ref().str_word)); }
  state.current().faliases = Some(faliases);
  state
}

pub fn cog_unaliasf(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(list) = state.current().stack.pop() else {
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let lstack = list.value_stack_ref();
  if lstack.len() == 0 || state.current().faliases.is_none() {
    state.pool.add_val(list);
    return state;
  }
  if lstack.iter().any(|x| !x.is_word()) {
    state.current().stack.push(list);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let mut faliases = state.current().faliases.take().unwrap();
  for v in lstack.iter() { faliases.remove(&v.vword_ref().str_word); }
  state.current().faliases = Some(faliases);
  state
}

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

pub fn cog_getd(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let v = if let Some(delims) = state.current().delims.take() {
    let mut v = state.pool.get_vword(delims.len());
    v.str_word.push_str(&delims);
    state.current().delims = Some(delims); v
  } else { state.pool.get_vword(0) };
  state.push_quoted(Value::Word(v)); state
}

pub fn cog_geti(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let v = if let Some(ignored) = state.current().ignored.take() {
    let mut v = state.pool.get_vword(ignored.len());
    v.str_word.push_str(&ignored);
    state.current().ignored = Some(ignored); v
  } else { state.pool.get_vword(0) };
  state.push_quoted(Value::Word(v)); state
}

pub fn cog_gets(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let v = if let Some(singlets) = state.current().singlets.take() {
    let mut v = state.pool.get_vword(singlets.len());
    v.str_word.push_str(&singlets);
    state.current().singlets = Some(singlets); v
  } else { state.pool.get_vword(0) };
  state.push_quoted(Value::Word(v)); state
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
  add_word!(state, "getf", cog_getf);
  add_word!(state, "setf", cog_setf);
  add_word!(state, "aliasf", cog_aliasf);
  add_word!(state, "unaliasf", cog_unaliasf);
  add_word!(state, "d", cog_d);
  add_word!(state, "i", cog_i);
  add_word!(state, "s", cog_s);
  add_word!(state, "dtgl", cog_dtgl);
  add_word!(state, "itgl", cog_itgl);
  add_word!(state, "stgl", cog_stgl);
  add_word!(state, "getd", cog_getd);
  add_word!(state, "geti", cog_geti);
  add_word!(state, "gets", cog_gets);
  add_word!(state, "delim", cog_delim);
  add_word!(state, "ignore", cog_ignore);
  add_word!(state, "singlet", cog_singlet);
}
