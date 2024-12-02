use cognition::*;
use std::thread;
use std::time::Duration;

pub fn cog_sleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(Duration::from_secs(i as u64));
  state
}
pub fn cog_msleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(Duration::from_millis(i as u64));
  state
}
pub fn cog_usleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(Duration::from_micros(i as u64));
  state
}
pub fn cog_nanosleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
  if i > u64::MAX as usize {
    return state.eval_error("OUT OF BOUNDS", w);
  } else {
    let v = state.current().stack.pop().unwrap();
    state.pool.add_val(v);
  }
  thread::sleep(Duration::from_nanos(i as u64));
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  add_word!(state, lib, "sleep", cog_sleep);
  add_word!(state, lib, "msleep", cog_msleep);
  add_word!(state, lib, "μsleep", cog_usleep);
  add_word!(state, lib, "nanosleep", cog_nanosleep);
}
