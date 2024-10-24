use crate::*;

pub fn cog_questionmark(state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let mut f = stdout();
  fwrite_check_pretty!(f, GRN);
  println!("STACK:");
  fwrite_check_pretty!(f, COLOR_RESET);
  let cur = state.current_ref();
  for v in cur.stack.iter() { v.print("\n"); }
  println!("");
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "?", cog_questionmark);
}
