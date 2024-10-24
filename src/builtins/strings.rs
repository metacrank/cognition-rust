use crate::*;

pub fn cog_cut(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  let int = match math.stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize > string.len() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w);
    } else {
      i as usize
    },
    Err(e) => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w);
    },
  };
  let mut new_word = state.pool.get_vword(string.len() - int);
  new_word.str_word.push_str(&string[int..]);
  string.truncate(int);

  state.current().stack.push(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "cut", cog_cut);
}
