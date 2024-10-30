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
  add_word!(state, "read", cog_read);
}
