use crate::*;
use std::io::*;

pub fn cog_questionmark(state: CognitionState, _: Option<&Value>) -> CognitionState {
  let mut f = stdout();
  fwrite_check_pretty!(f, GRN);
  println!("STACK:");
  fwrite_check_pretty!(f, COLOR_RESET);
  let cur = state.current_ref();
  for v in cur.stack.iter() { v.print("\n"); }
  println!("");
  state
}

pub fn cog_period(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  v.print("\n");
  state.pool.add_val(v);
  state
}

pub fn cog_print(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().iter().any(|x| !x.is_word()) {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  for wv in v.value_stack_ref().iter() {
    if stdout().write(wv.vword_ref().str_word.as_bytes()).is_err() {
      state = state.eval_error("FALIED WRITE", w);
      break
    }
  }
  state.pool.add_val(v);
  state
}

pub fn cog_read(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut vword = state.pool.get_vword(DEFAULT_STRING_LENGTH);
  if stdin().read_line(&mut vword.str_word).is_err() {
    state.pool.add_vword(vword);
    return state.eval_error("READ FAILED", w);
  }
  state.push_quoted(Value::Word(vword));
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "?", cog_questionmark);
  add_word!(state, ".", cog_period);
  add_word!(state, "print", cog_print);
  add_word!(state, "read", cog_read);
}
