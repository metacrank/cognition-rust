use crate::*;

pub fn cog_eclean(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let estack_o = state.current().err_stack.take();
  if let Some(mut estack) = estack_o {
    while let Some(v) = estack.pop() { state.pool.add_val(v) }
    state.current().err_stack = Some(estack);
  }
  state
}

pub fn cog_epeek(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if let Some(estack) = &mut state.current().err_stack {
    if let Some(v) = estack.pop() {
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
      state.current().err_stack.as_mut().unwrap().push(v);
      return state;
    }
  }
  cog_epop(state.eval_error("NO ERRORS", w), w)
}

pub fn cog_epop(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if let Some(estack) = &mut state.current().err_stack {
    if let Some(v) = estack.pop() {
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

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "epeek", cog_epeek);
  add_word!(state, "epop", cog_epop);
  add_word!(state, "epush", cog_epush);
}
