use crate::*;
use super::io::*;
use std::fs::File;

pub fn cog_eclean(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let estack_o = state.current().err_stack.take();
  if let Some(mut estack) = estack_o {
    while let Some(v) = estack.pop() { state.pool.add_val(v) }
    state.current().err_stack = Some(estack);
  }
  state
}

pub fn push_err_on_stack(state: &mut CognitionState, v: &Value) {
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
  match e.loc {
    Some(ref loc) => {
      let mut quot = state.pool.get_vstack(3);
      let mut v2 = state.pool.get_vword(loc.filename.len());
      v2.str_word.push_str(&loc.filename);
      quot.container.stack.push(Value::Word(v2));
      let mut v2 = state.pool.get_vword(loc.line.len());
      v2.str_word.push_str(&loc.line);
      quot.container.stack.push(Value::Word(v2));
      let mut v2 = state.pool.get_vword(loc.column.len());
      v2.str_word.push_str(&loc.column);
      quot.container.stack.push(Value::Word(v2));
      state.current().stack.push(Value::Stack(quot));
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
  if stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v3 = stack.pop().unwrap();
  let v2 = stack.pop().unwrap();
  let v1 = stack.pop().unwrap();
  let stack3 = v3.value_stack();
  let stack2 = v2.value_stack_ref();
  let stack1 = v1.value_stack_ref();
  if stack1.len() != 1 || stack2.len() > 1 || !(stack3.len() == 0 || stack3.len() == 3) {
    stack.push(v1);
    stack.push(v2);
    stack.push(v3);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let w1 = stack1.first().unwrap();
  let w2 = stack2.first();
  if stack3.iter().any(|x| !x.is_word()) { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Value::Word(ref v1word) = w1 else { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let str_word = match w2 {
    Some(Value::Word(ref v2word)) => Some(state.string_copy(&v2word.str_word)),
    None => None,
    _ => return state.eval_error("BAD ARGUMENT TYPE", w),
  };
  let mut v = state.pool.get_verror(v1word.str_word.len());
  v.error.push_str(&v1word.str_word);
  v.str_word = str_word;
  if stack3.len() == 3 {
    let mut loc = state.pool.get_verror_loc(0);
    std::mem::swap(&mut stack3.first_mut().as_mut().unwrap().vword_mut().str_word, &mut loc.filename);
    std::mem::swap(&mut stack3.get_mut(1).as_mut().unwrap().vword_mut().str_word, &mut loc.line);
    std::mem::swap(&mut stack3.get_mut(2).as_mut().unwrap().vword_mut().str_word, &mut loc.column);
    v.loc = Some(loc)
  }
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  state.pool.add_val(v3);
  if state.current().err_stack.is_none() {
    state.current().err_stack = Some(state.pool.get_stack(1));
  }
  state.current().err_stack.as_mut().unwrap().push(Value::Error(v));
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

pub fn cog_eprint(state: CognitionState, w: Option<&Value>) -> CognitionState {
  let err_stack = &state.current_ref().err_stack;
  if let Some(estack) = err_stack {
    let e = estack.last();
    if let Some(v) = e {
      v.print("\n");
      return state;
    }
  }
  state.eval_error("NO ERRORS", w)
}

pub fn cog_fewrite(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(mut v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let err_stack = &state.current_ref().err_stack;
  let Some(estack) = err_stack else {
    state.current().stack.push(v);
    return state.eval_error("NO ERRORS", w)
  };
  if estack.last().is_none() {
    state.current().stack.push(v);
    return state.eval_error("NO ERRORS", w)
  }

  match v.value_stack().first_mut().unwrap() {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let file = file.file.as_mut().unwrap();
        let is_terminal = file.is_terminal();
        estack.last().as_ref().unwrap().fprint(file, "\n", is_terminal);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        estack.last().as_ref().unwrap().fprint(writer.writer.as_mut().unwrap(), "\n", false);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        estack.last().as_ref().unwrap().fprint(stream.stream.as_mut().unwrap(), "\n", false);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        estack.last().as_ref().unwrap().fprint(bufwriter.bufwriter.as_mut().unwrap(), "\n", false);
      } else {
        state.current().stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::create(&vword.str_word) {
        let is_terminal = file.is_terminal();
        estack.last().as_ref().unwrap().fprint(&mut file, "\n", is_terminal);
      } else {
        state.current().stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      state.current().stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_feprint(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(mut v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let err_stack = &state.current_ref().err_stack;
  let Some(estack) = err_stack else {
    state.current().stack.push(v);
    return state.eval_error("NO ERRORS", w)
  };
  if estack.last().is_none() {
    state.current().stack.push(v);
    return state.eval_error("NO ERRORS", w)
  }

  match v.value_stack().first_mut().unwrap() {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        let file = file.file.as_mut().unwrap();
        let is_terminal = file.is_terminal();
        estack.last().as_ref().unwrap().fprint(file, "\n", is_terminal);
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        estack.last().as_ref().unwrap().fprint(writer.writer.as_mut().unwrap(), "\n", false);
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        estack.last().as_ref().unwrap().fprint(stream.stream.as_mut().unwrap(), "\n", false);
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        estack.last().as_ref().unwrap().fprint(bufwriter.bufwriter.as_mut().unwrap(), "\n", false);
      } else {
        state.current().stack.push(v);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::options().append(true).create(true).open(&vword.str_word) {
        let is_terminal = file.is_terminal();
        estack.last().as_ref().unwrap().fprint(&mut file, "\n", is_terminal);
      } else {
        state.current().stack.push(v);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v);
    },
    _ => return {
      state.current().stack.push(v);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

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
  e.loc = state.verr_loc();
  if state.current().err_stack.is_none() {
    state.current().err_stack = Some(state.pool.get_stack(1));
  }
  state.current().err_stack.as_mut().unwrap().push(Value::Error(e));
  state
}

pub fn cog_esize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let length = if let Some(ref e) = state.current_ref().err_stack { e.len() } else { 0 };
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  if length > isize::MAX as usize { return state.with_math(math).eval_error("OUT OF BOUNDS", w) }
  match math.math().itos(length as isize, &mut state) {
    Ok(s) => {
      state.set_math(math);
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => { return state.with_math(math).eval_error(e, w) }
  }
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "eclean", cog_eclean);
  add_builtin!(state, "epeek", cog_epeek);
  add_builtin!(state, "epop", cog_epop);
  add_builtin!(state, "epush", cog_epush);
  add_builtin!(state, "edrop", cog_edrop);
  add_builtin!(state, "eprint", cog_eprint);
  add_builtin!(state, "fewrite", cog_fewrite);
  add_builtin!(state, "feprint", cog_feprint);
  add_builtin!(state, "eshow", cog_eshow);
  add_builtin!(state, "ethrow", cog_ethrow);
  add_builtin!(state, "esize", cog_esize);
}
