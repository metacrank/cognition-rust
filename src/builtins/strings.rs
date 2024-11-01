use crate::*;

pub fn cog_concat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v2 = stack.last().unwrap();
  let v1 = stack.get(stack.len() - 2).unwrap();
  if v1.value_stack_ref().len() == 0 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let mut concatlen: usize = 0;
  for v in v1.value_stack_ref().iter() {
    let Value::Word(vw) = v else { return state.eval_error("BAD ARGUMENT TYPE", w) };
    concatlen += vw.str_word.len()
  }
  for v in v2.value_stack_ref().iter() {
    let Value::Word(vw) = v else { return state.eval_error("BAD ARGUMENT TYPE", w) };
    concatlen += vw.str_word.len()
  }
  let mut new_word = state.pool.get_vword(concatlen);
  let mut iterate = |v: &Value| for val in v.value_stack_ref().iter() {
    new_word.str_word.push_str(&val.vword_ref().str_word)
  };
  let v2 = state.current().stack.pop().unwrap();
  let v1 = state.current().stack.pop().unwrap();
  iterate(&v1);
  iterate(&v2);
  state.pool.add_val(v2);
  state.pool.add_val(v1);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_unconcat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let mut unconcatlen: usize = 0;
  for val in v.value_stack_ref().iter() {
    let Value::Word(word) = val else {
      stack.push(v);
      return state.eval_error("BAD ARGUMENT TYPE", w)
    };
    unconcatlen += word.str_word.len()
  }
  let mut newstack = state.pool.get_vstack(unconcatlen);
  for val in v.value_stack_ref().iter() {
    let string = &val.vword_ref().str_word;
    for c in string.chars() {
      let mut vword = state.pool.get_vword(1);
      vword.str_word.push(c);
      newstack.container.stack.push(Value::Word(vword))
    }
  }
  state.pool.add_val(v);
  state.current().stack.push(Value::Stack(newstack));
  state
}

pub fn cog_cut(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(vint) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(mut vstr) = stack.pop() else {
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Some(ref math) = state.current_ref().math else {
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let string = &mut vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let int = match math.stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize > string.len() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      i as usize
    },
    Err(e) => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w)
    },
  };
  if !string.is_char_boundary(int) {
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("INVALID CHAR BOUNDARY", w)
  }
  state.pool.add_val(vint);
  let mut new_word = state.pool.get_vword(string.len() - int);
  new_word.str_word.push_str(&string[int..]);
  string.truncate(int);

  state.current().stack.push(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_len(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  let Some(v) = cur.stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = v_stack.first().unwrap();
  if !word_v.is_word() { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Some(ref math) = cur.math else { return state.eval_error("MATH BASE ZERO", w) };
  if math.base() == 0 { return state.eval_error("MATH BASE ZERO", w) }
  let length = word_v.vword_ref().str_word.len();
  if length > isize::MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
  match math.itos(word_v.vword_ref().str_word.len() as isize, &mut state) {
    Ok(s) => {
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state = state.push_cur(cur_v);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => return state.push_cur(cur_v).eval_error(e, w),
  }
}

pub fn cog_cat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut vint = stack.pop().unwrap();
  let vstr = stack.last().unwrap();
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Some(ref math) = state.current_ref().math else {
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let vstr_stack = state.current_ref().stack.last().unwrap().value_stack_ref();
  let string = &vstr_stack.first().unwrap().vword_ref().str_word;
  let int = match math.stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize >= string.len() {
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      i as usize
    },
    Err(e) => {
      state.current().stack.push(vint);
      return state.eval_error(e, w)
    },
  };
  if !string.is_char_boundary(int) {
    state.current().stack.push(vint);
    return state.eval_error("INVALID CHAR BOUNDARY", w)
  }
  let vstr_stack = state.current_ref().stack.last().unwrap().value_stack_ref();
  let string = &vstr_stack.first().unwrap().vword_ref().str_word;
  let s = &mut vint.value_stack().first_mut().unwrap().vword_mut().str_word;
  s.clear();
  s.push(string[int..].chars().next().unwrap().clone());
  state.current().stack.push(vint);
  state
}

pub fn cog_insert(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(vint) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(vstr) = stack.pop() else {
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  let Some(vsrc) = stack.pop() else {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 || vsrc.value_stack_ref().len() != 1 {
    stack.push(vsrc);
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() || !vsrc.value_stack_ref().first().unwrap().is_word() {
    stack.push(vsrc);
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Some(ref math) = state.current_ref().math else {
    state.current().stack.push(vsrc);
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let string = &vstr.value_stack_ref().first().unwrap().vword_ref().str_word;
  let source = &vsrc.value_stack_ref().first().unwrap().vword_ref().str_word;
  let int = match math.stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize > source.len() {
      state.current().stack.push(vsrc);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      i as usize
    },
    Err(e) => {
      state.current().stack.push(vsrc);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w)
    },
  };
  if !source.is_char_boundary(int) {
    state.current().stack.push(vsrc);
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("INVALID CHAR BOUNDARY", w)
  }
  let mut new_word = state.pool.get_vword(string.len() + source.len());
  new_word.str_word.push_str(&source[..int]);
  new_word.str_word.push_str(string);
  new_word.str_word.push_str(&source[int..]);

  state.pool.add_val(vint);
  state.pool.add_val(vstr);
  state.pool.add_val(vsrc);

  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_reverse(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Value::Word(vw) = v.value_stack_ref().first().unwrap() else {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let mut new_vword = state.pool.get_vword(vw.str_word.len());
  for c in vw.str_word.chars().rev() {
    new_vword.str_word.push(c);
  }
  state.pool.add_val(v.value_stack().pop().unwrap());
  v.value_stack().push(Value::Word(new_vword));
  state.current().stack.push(v);
  state
}

pub fn cog_word_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let mut vword = state.pool.get_vword(1);
  if v.value_stack_ref().len() == 1 {
    if v.value_stack_ref().first().unwrap().is_word() {
      vword.str_word.push('t') }}
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_ctoi(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let char_v = v.value_stack().first_mut().unwrap();
  let Value::Word(vword_char) = char_v else {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let mut chars = vword_char.str_word.chars();
  let c = chars.next();
  if c.is_none() || chars.next().is_some() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(math) = state.current().math.take() else {
    state.current().stack.push(v);
    return state.eval_error("MATH BASE ZERO", w)
  };
  match math.itos(c.unwrap() as isize, &mut state) {
    Ok(mut s) => {
      state.current().math = Some(math);
      std::mem::swap(&mut vword_char.str_word, &mut s);
      state.pool.add_string(s);
      state.current().stack.push(v);
      state
    },
    Err(e) => {
      state.current().math = Some(math);
      state.current().stack.push(v);
      state.eval_error(e, w)
    }
  }
}

pub fn cog_itoc(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_int!(state, w, i32, ACTIVE, "INVALID CHAR VALUE");
  let Some(c) = char::from_u32(i as u32) else {
    return state.eval_error("INVALID CHAR VALUE", w)
  };
  let vword = state.current().stack.last_mut().unwrap().value_stack().first_mut().unwrap().vword_mut();
  vword.str_word.clear();
  vword.str_word.push(c);
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "concat", cog_concat);
  add_word!(state, "unconcat", cog_unconcat);
  add_word!(state, "cut", cog_cut);
  add_word!(state, "len", cog_len);
  add_word!(state, "cat", cog_cat);
  add_word!(state, "insert", cog_insert);
  add_word!(state, "reverse", cog_reverse);
  add_word!(state, "word?", cog_word_questionmark);
  add_word!(state, "ctoi", cog_ctoi);
  add_word!(state, "itoc", cog_itoc);
}
