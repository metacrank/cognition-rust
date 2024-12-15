use cognition::*;
use std::thread;
use std::time::Duration;
use time::DurationCustom;

pub fn cog_sleep(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) };
  let duration = match v.value_stack_ref().first().unwrap() {
    Value::Custom(vcustom) => match unsafe { vcustom.custom.as_custom_ref::<DurationCustom>() } {
      Some(d) => if d.neg { Duration::ZERO } else { d.duration.clone() },
      None => return state.eval_error("BAD ARGUMENT TYPE", w)
    },
    _ => {
      let i = get_unsigned!(state, w, isize, ACTIVE) as usize;
      if i > u64::MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
      Duration::from_secs(i as u64)
    }
  };
  let v = state.current().stack.pop().unwrap();
  state.pool.add_val(v);
  thread::sleep(duration);
  state
}



#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  add_word!(state, lib, "sleep", cog_sleep);
}
