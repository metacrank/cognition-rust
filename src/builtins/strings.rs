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
  let Some(ref math) = state.current().math else {
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

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "concat", cog_concat);
  add_word!(state, "cut", cog_cut);
  add_word!(state, "len", cog_len);
}
