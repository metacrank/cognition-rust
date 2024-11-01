#![allow(unused_imports)]
use cognition::*;
use unicode_segmentation::Graphemes;
use unicode_segmentation::UnicodeSegmentation;

pub fn cog_gcut(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(vint) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(mut vstr) = stack.pop() else {
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w);
  };
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let Some(ref math) = state.current().math else {
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w);
  };
  let string = &mut vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let mut iter = string.grapheme_indices(true);
  match math.stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(mut i) => if i < 0 || i.abs() as usize > string.len() {
        state.current().stack.push(vstr);
        state.current().stack.push(vint);
        return state.eval_error("OUT OF BOUNDS", w)
      } else {
        loop {
          if i == 0 { break }
          if iter.next().is_none() {
            state.current().stack.push(vstr);
            state.current().stack.push(vint);
            return state.eval_error("OUT OF BOUNDS", w)
          }
          i -= 1
        }
      },
    Err(e) => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w);
    },
  };
  let new_word = match iter.next() {
    Some((i, _)) => {
      let mut new_word = state.pool.get_vword(string.len() - i);
      new_word.str_word.push_str(&string[i..]);
      string.truncate(i);
      new_word
    },
    None => state.pool.get_vword(0),
  };
  state.current().stack.push(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_gunconcat(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_gcat(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_glen(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut cur_v = state.pop_cur();
  let cur = cur_v.metastack_container();
  let Some(v) = cur.stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = v_stack.first().unwrap();
  if !word_v.is_word() { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let Some(ref math) = cur.math else { return state.eval_error("MATH BASE ZERO", w) };
  if math.base() == 0 { return state.eval_error("MATH BASE ZERO", w) }
  let mut length: usize = 0;
  for _ in word_v.vword_ref().str_word.graphemes(true) { length += 1 }
  if math.base() == 1 && length != 0 {
    return state.eval_error("MATH BASE ONE", w)
  }
  if length > isize::MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
  match math.itos(length as isize, &mut state) {
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

pub fn cog_ginsert(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_greverse(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_gtoi(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

pub fn cog_itog(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState) {
  add_word!(state, "gcut", cog_gcut);
  add_word!(state, "glen", cog_glen);
}
