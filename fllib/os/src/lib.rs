use cognition::*;
use std::env::{var,set_var,remove_var};

pub fn cog_getenv(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut vw = get_word!(state, w);
  let vword = vw.value_stack_ref().first().unwrap().vword_ref();
  match var(&vword.str_word) {
    Ok(val) => {
      let mut vword = state.pool.get_vword(val.len());
      vword.str_word.push_str(&val);
      state.pool.add_val(vw);
      state.push_quoted(Value::Word(vword));
    },
    Err(_) => {
      let v = vw.value_stack().pop().unwrap();
      state.pool.add_val(v);
      state.current().stack.push(vw);
    }
  }
  state
}

pub fn cog_setenv(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let (v1, v2) = get_2_words!(state, w);
  let vword1 = v1.value_stack_ref().first().unwrap().vword_ref();
  let vword2 = v2.value_stack_ref().first().unwrap().vword_ref();
  set_var(&vword1.str_word, &vword2.str_word);
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  state
}

pub fn cog_rmenv(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vw = get_word!(state, w);
  let vword = vw.value_stack_ref().first().unwrap().vword_ref();
  remove_var(&vword.str_word);
  state.pool.add_val(vw);
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState) {
  add_word!(state, "getenv", cog_getenv);
  add_word!(state, "setenv", cog_setenv);
  add_word!(state, "rmenv", cog_rmenv);
}
