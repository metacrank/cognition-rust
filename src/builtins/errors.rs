use crate::*;

pub fn cog_eclean(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let estack_o = state.current().err_stack.take();
  if let Some(mut estack) = estack_o {
    while let Some(v) = estack.pop() { state.pool.add_val(v) }
    state.current().err_stack = Some(estack);
  }
  state
}

fn push_err_on_stack(state: &mut CognitionState, v: &Value) {
  let e = v.verror_ref();
  let mut v1 = state.pool.get_vword(e.error.len());
  v1.str_word.push_str(&e.error);
  state.push_quoted(Value::Word(v1));
  match e.str_word {
    Some(ref s) => {
      let mut v2 = state.pool.get_vword(s.len());
      v2.str_word.push_str(s);
      state.push_quoted(Value::Word(v2));
    },
    None => {
      let v2 = state.pool.get_vstack(0);
      state.current().stack.push(Value::Stack(v2));
    },
  }
}

pub fn cog_epeek(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if let Some(estack) = &mut state.current().err_stack {
    if let Some(v) = estack.pop() {
      push_err_on_stack(&mut state, &v);
      state.current().err_stack.as_mut().unwrap().push(v);
      return state;
    }
  }
  cog_epop(state.eval_error("NO ERRORS", w), w)
}

pub fn cog_epop(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if let Some(estack) = &mut state.current().err_stack {
    if let Some(v) = estack.pop() {
      push_err_on_stack(&mut state, &v);
      state.pool.add_val(v);
      return state;
    }
  }
  cog_epop(state.eval_error("NO ERRORS", w), w)
}

pub fn cog_epush(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v2) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let stack2 = v2.value_stack_ref();
  let stack1 = v1.value_stack_ref();
  if stack1.len() != 1 || stack2.len() > 1 {
    stack.push(v1);
    stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let w1 = stack1.first().unwrap();
  let w2 = stack2.first();
  if w2.is_some() {
    let Some(Value::Word(_)) = w2 else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  }
  let Value::Word(ref v1word) = w1 else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let mut v = state.pool.get_verror(v1word.str_word.len());
  v.error.push_str(&v1word.str_word);
  v.str_word = match w2 {
    Some(Value::Word(ref v2word)) => Some(state.string_copy(&v2word.str_word)),
    None => None,
    _ => unreachable!(),
  };
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  let mut err_stack = state.current().err_stack.take();
  match &mut err_stack {
    Some(estack) => estack.push(Value::Error(v)),
    None => {
      err_stack = Some(state.pool.get_stack(1));
      err_stack.as_mut().unwrap().push(Value::Error(v))
    },
  }
  state.current().err_stack = err_stack.take();
  state
}

pub fn cog_edrop(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let err_stack = &mut state.current().err_stack;
  if let Some(estack) = err_stack {
    let e = estack.pop();
    if let Some(v) = e {
      state.pool.add_val(v);
      return state;
    }
  }
  state.eval_error("NO ERRORS", w)
}

pub fn cog_eprint(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let err_stack = &mut state.current().err_stack;
  if let Some(estack) = err_stack {
    let e = estack.last();
    if let Some(v) = e {
      v.print("\n");
      return state;
    }
  }
  state.eval_error("NO ERRORS", w)
}

// cog_feprint

pub fn cog_eshow(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  println!("Error stack:");
  let err_stack = &mut state.current().err_stack;
  if let Some(estack) = err_stack {
    for v in estack.iter() {
      v.print("\n");
    }
  }
  state
}

pub fn cog_ethrow(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(mut v) = cur.stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let err_stack = v.value_stack();
  if err_stack.len() != 1 {
    cur.stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let err_v = err_stack.first().unwrap();
  let Value::Word(err_w) = err_v else {
    cur.stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  };
  let mut e = state.pool.get_verror(err_w.str_word.len());
  e.error.push_str(&err_w.str_word);
  if let Some(Value::Word(vw)) = w { e.str_word = Some(state.string_copy(&vw.str_word)); }
  state.pool.add_val(v);
  if state.current().err_stack.is_none() {
    state.current().err_stack = Some(state.pool.get_stack(DEFAULT_STACK_SIZE));
  }
  state.current().err_stack.as_mut().unwrap().push(Value::Error(e));
  state
}

pub fn cog_esize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  if cur.math.is_none() { return state.push_cur(cur_v).eval_error("MATH BASE UNINITIALIZED", w) }
  if cur.math.as_ref().unwrap().base() == 0 { return state.push_cur(cur_v).eval_error("MATH BASE ZERO", w) }
  if cur.math.as_ref().unwrap().base() == 1 { return state.push_cur(cur_v).eval_error("MATH BASE ONE", w) }
  let length = if let Some(ref e) = cur.err_stack { e.len() } else { 0 };
  if length > isize::MAX as usize { return state.push_cur(cur_v).eval_error("OUT OF BOUNDS", w) }
  match cur.math.as_ref().unwrap().itos(length as isize, &mut state) {
    Ok(s) => {
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state = state.push_cur(cur_v);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => { return state.push_cur(cur_v).eval_error(e, w) }
  }
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "eclean", cog_eclean);
  add_word!(state, "epeek", cog_epeek);
  add_word!(state, "epop", cog_epop);
  add_word!(state, "epush", cog_epush);
  add_word!(state, "edrop", cog_edrop);
  add_word!(state, "eprint", cog_eprint);
  // cog_feprint
  add_word!(state, "eshow", cog_eshow);
  add_word!(state, "ethrow", cog_ethrow);
  add_word!(state, "esize", cog_esize);
}
