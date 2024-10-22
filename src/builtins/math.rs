use crate::*;
use crate::math::BASE_MAX;

macro_rules! ensure_math {
  ($state:ident) => {
    if $state.current_ref().math.is_none() {
      $state.current().math = Some($state.pool.get_math(0))
    }
  };
}

pub fn cog_base(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v) = cur.stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(math) = &mut cur.math else { return state.eval_error("MATH DIGITS UNINITIALIZED", w) };
  let i = if math.base() == 0 {
    let i = v.value_stack_ref().len();
    if i > BASE_MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
    i as i32
  } else {
    if v.value_stack_ref().len() != 1 {
      return state.eval_error("BAD ARGUMENT TYPE", w)
    }
    let base_v = &v.value_stack_ref()[0];
    if !base_v.is_word() {
      return state.eval_error("BAD ARGUMENT TYPE", w)
    }
    match math.stoi(&base_v.vword_ref().str_word) {
      Ok(i) => if i > BASE_MAX as isize || i < 0 {
        return state.eval_error("MATH DIGITS UNINITIALIZED", w);
      } else { i as i32 },
      Err(e) => return state.eval_error(e, w),
    }
  };
  if let Some(e) = math.set_base(i) {
    return state.eval_error(e, w)
  }
  let v = cur.stack.pop().unwrap();
  state.pool.add_val(v);
  state
}

pub fn cog_negc(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char!(state, c, w);
  ensure_math!(state);
  state.current().math.as_mut().unwrap().set_negc(c);
  state
}
pub fn cog_radix(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char!(state, c, w);
  ensure_math!(state);
  state.current().math.as_mut().unwrap().set_radix(c);
  state
}
// Cayley Dickson delimiter
pub fn cog_cd_delim(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  get_char!(state, c, w);
  ensure_math!(state);
  state.current().math.as_mut().unwrap().set_delim(c);
  state
}

pub fn cog_digits(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  let Some(v) = cur.stack.last() else { return state.push_cur(cur_v).eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.push_cur(cur_v).eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = &v.value_stack_ref()[0];
  if !word_v.is_word() { return state.push_cur(cur_v).eval_error("BAD ARGUMENT TYPE", w) }
  let s = &word_v.vword_ref().str_word;
  if s.len() > i32::MAX as usize { return state.push_cur(cur_v).eval_error("OUT OF BOUNDS", w) }
  if cur.math.is_none() {
    cur.math = Some(state.pool.get_math(0))
  }
  cur.math.as_mut().unwrap().set_digits(s);
  let v = cur.stack.pop().unwrap();
  state.pool.add_val(v);
  state.push_cur(cur_v)
}

pub fn cog_plus(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let cur = state.current();
  let Some(v2) = cur.stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(v1) = cur.stack.pop() else {
    cur.stack.push(v2);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  if cur.math.is_none() {
    cur.stack.push(v1);
    cur.stack.push(v2);
    return state.eval_error("MATH BASE ZERO", w)
  }
  if cur.math.as_ref().unwrap().base() == 0 {
    cur.stack.push(v1);
    cur.stack.push(v2);
    return state.eval_error("MATH BASE ZERO", w)
  }
  if v1.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
    cur.stack.push(v1);
    cur.stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let word_v1 = &v1.value_stack_ref()[0];
  let word_v2 = &v2.value_stack_ref()[0];
  if !word_v1.is_word() || !word_v2.is_word() {
    cur.stack.push(v1);
    cur.stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let s1 = &word_v1.vword_ref().str_word;
  let s2 = &word_v1.vword_ref().str_word;
  if s1.len() > i32::MAX as usize || s2.len() > i32::MAX as usize {
    cur.stack.push(v1);
    cur.stack.push(v2);
    return state.eval_error("OUT OF BOUNDS", w)
  }

  let n1 = match cur.math.as_ref().unwrap().stoi(s1) {
    Ok(i) => i,
    Err(e) => {
      cur.stack.push(v1);
      cur.stack.push(v2);
      return state.eval_error(e, w)
    }
  };
  let n2 = match cur.math.as_ref().unwrap().stoi(s2) {
    Ok(i) => i,
    Err(e) => {
      cur.stack.push(v1);
      cur.stack.push(v2);
      return state.eval_error(e, w)
    }
  };
  let math = cur.math.take().unwrap();
  match math.itos(n1 + n2, &mut state) {
    Ok(s) => {
      let mut vw = state.pool.get_vword(s.len());
      vw.str_word.push_str(&s);
      state.pool.add_string(s);
      state.pool.add_val(v1);
      state.pool.add_val(v2);
      state.push_quoted(Value::Word(vw));
      state
    },
    Err(e) => {
      state.current().math = Some(math);
      state.current().stack.push(v1);
      state.current().stack.push(v2);
      state.eval_error(e, w)
    }
  }
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "base", cog_base);
  add_word!(state, "negc", cog_negc);
  add_word!(state, "radix", cog_radix);
  add_word!(state, "cd-delim", cog_cd_delim);
  add_word!(state, "digits", cog_digits);
  add_word!(state, "+", cog_plus);
}
