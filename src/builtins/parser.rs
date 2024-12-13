use crate::*;

pub fn cog_getf(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
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

pub fn cog_f(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
    state.current().faliases = Some(state.pool.get_faliases(DEFAULT_FALIASES_SIZE));
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
    state.current().faliases = Some(state.pool.get_faliases(DEFAULT_FALIASES_SIZE));
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

pub fn cog_dtgl(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  state.current().dflag = !state.current_ref().dflag;
  state
}

pub fn cog_itgl(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  state.current().iflag = !state.current_ref().iflag;
  state
}

pub fn cog_stgl(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  state.current().sflag = !state.current_ref().sflag;
  state
}

pub fn cog_dflag(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vword = if state.current_ref().dflag {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('w');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_iflag(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vword = if state.current_ref().iflag {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('w');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_sflag(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vword = if state.current_ref().sflag {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('w');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_getd(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let v = if let Some(delims) = state.current().delims.take() {
    let mut v = state.pool.get_vword(delims.len());
    v.str_word.push_str(&delims);
    state.current().delims = Some(delims); v
  } else { state.pool.get_vword(0) };
  state.push_quoted(Value::Word(v)); state
}

pub fn cog_geti(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let v = if let Some(ignored) = state.current().ignored.take() {
    let mut v = state.pool.get_vword(ignored.len());
    v.str_word.push_str(&ignored);
    state.current().ignored = Some(ignored); v
  } else { state.pool.get_vword(0) };
  state.push_quoted(Value::Word(v)); state
}

pub fn cog_gets(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
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

pub fn cog_undelim(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  if let Some(delims) = &mut state.current().delims {
    for c in vword.str_word.chars() { delims.retain(|x| x != c); }
  }
  state.pool.add_val(v);
  state
}

pub fn cog_unignore(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  if let Some(ignored) = &mut state.current().ignored {
    for c in vword.str_word.chars() { ignored.retain(|x| x != c); }
  }
  state.pool.add_val(v);
  state
}

pub fn cog_unsinglet(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  if let Some(singlets) = &mut state.current().singlets {
    for c in vword.str_word.chars() { singlets.retain(|x| x != c); }
  }
  state.pool.add_val(v);
  state
}

pub fn cog_filename(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(parser) = &mut state.parser else { return state.eval_error("NO PARSER", w) };
  let Some(filename) = parser.filename.take() else { return state.eval_error("NO FILENAME", w) };
  let mut vword = state.pool.get_vword(filename.len());
  vword.str_word.push_str(&filename);
  state.parser.as_mut().unwrap().filename = Some(filename);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_line(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(ref parser) = state.parser else { return state.eval_error("NO PARSER", w) };
  if parser.filename.is_none() { return state.eval_error("NO FILENAME", w) }
  let line = parser.line;
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  if line > isize::MAX as usize { return state.with_math(math).eval_error("OUT OF BOUNDS", w) }
  match math.math().itos(line as isize, &mut state) {
    Ok(s) => {
      state.set_math(math);
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => state.with_math(math).eval_error(e, w)
  }
}

pub fn cog_column(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(ref parser) = state.parser else { return state.eval_error("NO PARSER", w) };
  if parser.filename.is_none() { return state.eval_error("NO FILENAME", w) }
  let column = parser.column;
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  if column > isize::MAX as usize { return state.with_math(math).eval_error("OUT OF BOUNDS", w) }
  match math.math().itos(column as isize, &mut state) {
    Ok(s) => {
      state.set_math(math);
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => state.with_math(math).eval_error(e, w)
  }
}

pub fn cog_streval(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(mut v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let stack = v.value_stack_ref();
  if !(stack.len() == 1 || stack.len() == 2 || stack.len() == 4) {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if stack.iter().any(|x| !x.is_word()) {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let parser_loc = if stack.len() == 4 {
    let filename = &stack.get(1).unwrap().vword_ref().str_word;
    ParserLoc{ filename: Some(state.string_copy(filename)), pos: None }
  } else if stack.len() == 2 {
    let filename = &stack.get(1).unwrap().vword_ref().str_word;
    ParserLoc{ filename: Some(state.string_copy(filename)), pos: None }
  } else { ParserLoc{ filename: None, pos: None } };
  let val = if let Value::Stack(vstack) = &mut v {
    let mut new_vstack = state.pool.get_vstack(0);
    std::mem::swap(&mut new_vstack.container.stack, &mut vstack.container.stack);
    let wd = state.pool.get_word_def(v);
    state.family.push(wd);
    Value::Stack(new_vstack)
  } else { v };
  let v = val.value_stack_ref().first().unwrap();
  let oldparser = state.parser.take();
  state.parser = Some(Parser::with_loc(Some(state.string_copy(&v.vword_ref().str_word)), parser_loc));
  loop {
    let vw = state.parser_get_next();
    match vw {
      Some(v) => state = state.eval(v, w),
      None => break,
    }
    if state.exited { break }
    if state.control.is_return() {
      state.control.clear();
      break
    }
  }
  let parser = state.parser.take();
  state.parser = oldparser;
  if let Some(parser) = parser { state.pool.add_parser(parser) }
  if val.is_stack() {
    if let Some(wd) = state.family.pop() {
      state.pool.add_word_def(wd) }}
  state.pool.add_val(val);
  state
}

pub fn cog_strstack(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let stack = v.value_stack_ref();
  if stack.len() == 0 {
    state.pool.add_val(v);
    return state;
  }
  if stack.iter().any(|x| !x.is_word()) {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let mut quot = state.pool.get_vstack(DEFAULT_STACK_SIZE);
  for v in stack.iter() {
    let mut parser = Parser::new(Some(state.string_copy(&v.vword_ref().str_word)), None);
    loop {
      let w = parser.get_next(&mut state);
      match w {
        Some(v) => quot.container.stack.push(v),
        None => break,
      }
    }
  }
  state.current().stack.push(Value::Stack(quot));
  state.pool.add_val(v);
  state
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "getf", cog_getf);
  add_builtin!(state, "f", cog_f);
  add_builtin!(state, "aliasf", cog_aliasf);
  add_builtin!(state, "unaliasf", cog_unaliasf);
  add_builtin!(state, "d", cog_d);
  add_builtin!(state, "i", cog_i);
  add_builtin!(state, "s", cog_s);
  add_builtin!(state, "dtgl", cog_dtgl);
  add_builtin!(state, "itgl", cog_itgl);
  add_builtin!(state, "stgl", cog_stgl);
  add_builtin!(state, "dflag", cog_dflag);
  add_builtin!(state, "iflag", cog_iflag);
  add_builtin!(state, "sflag", cog_sflag);
  add_builtin!(state, "getd", cog_getd);
  add_builtin!(state, "geti", cog_geti);
  add_builtin!(state, "gets", cog_gets);
  add_builtin!(state, "delim", cog_delim);
  add_builtin!(state, "ignore", cog_ignore);
  add_builtin!(state, "singlet", cog_singlet);
  add_builtin!(state, "undelim", cog_undelim);
  add_builtin!(state, "unignore", cog_unignore);
  add_builtin!(state, "unsinglet", cog_unsinglet);
  add_builtin!(state, "filename", cog_filename);
  add_builtin!(state, "line", cog_line);
  add_builtin!(state, "column", cog_column);
  add_builtin!(state, "streval", cog_streval);
  add_builtin!(state, "strstack", cog_strstack);
}
