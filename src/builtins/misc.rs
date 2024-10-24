use crate::*;

pub fn cog_nop(state: CognitionState, _w: Option<&Value>) -> CognitionState { state }

pub fn cog_getargs(mut state: CognitionState, _w: Option<&Value>) -> CognitionState {
  let mut vstack = state.pool.get_vstack(state.args.len());
  for s in state.args.iter() {
    let mut vword = state.pool.get_vword(s.len());
    vword.str_word.push_str(&s);
    vstack.container.stack.push(Value::Word(vword));
  }
  state.push_quoted(Value::Stack(vstack));
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "nothing");
  add_word!(state, "return", RETURN);
  add_word!(state, "nop", cog_nop);
  add_word!(state, "getargs", cog_getargs);
}
