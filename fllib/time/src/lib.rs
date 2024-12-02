pub mod custom;

pub use crate::custom::*;
use cognition::*;
use std::time::{Duration, Instant};

macro_rules! duration_from {
  ($state:ident,$w:ident,$from:expr) => {
    let i = get_unsigned!($state, $w, isize, ACTIVE, "DURATION UNDERFLOW") as usize;
    if i > u64::MAX as usize { return $state.eval_error("DURATION OVERFLOW", $w) }
    let v = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v);
    let vcustom = get_duration_custom(&mut $state.pool, $from(i as u64));
    $state.push_quoted(Value::Custom(vcustom));
    $state
  }
}
macro_rules! duration_as {
  ($state:ident,$w:ident,$to:tt) => {
    let v = get_custom!($state, $w);
    let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
    let Some(duration_custom) = custom.downcast_ref::<DurationCustom>() else {
      $state.current().stack.push(v);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(math) = $state.get_math() else {
      $state.current().stack.push(v);
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.math().base() == 0 {
      $state.set_math(math);
      $state.current().stack.push(v);
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i = duration_custom.duration.$to() as u128;
    if i > isize::MAX as u128 {
      $state.set_math(math);
      $state.current().stack.push(v);
      return $state.eval_error("INTEGER OVERFLOW", $w)
    }
    let mut s = match math.math().itos(i as isize, &mut $state) {
      Ok(s) => s,
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v);
        return $state.eval_error(e, $w)
      }
    };
    $state.set_math(math);
    $state.pool.add_val(v);
    let mut vword = $state.pool.get_vword(0);
    std::mem::swap(&mut vword.str_word, &mut s);
    $state.pool.add_string(s);
    $state.push_quoted(Value::Word(vword));
    $state
  }
}

pub fn cog_nanos(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_from!{state, w, Duration::from_nanos}
}
pub fn cog_micros(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_from!{state, w, Duration::from_micros}
}
pub fn cog_millis(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_from!{state, w, Duration::from_millis}
}
pub fn cog_secs(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_from!{state, w, Duration::from_secs}
}

pub fn cog_as_nanos(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_as!{state, w, as_nanos}
}
pub fn cog_as_micros(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_as!{state, w, as_micros}
}
pub fn cog_as_millis(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_as!{state, w, as_millis}
}
pub fn cog_as_secs(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  duration_as!{state, w, as_secs}
}

pub fn cog_add_duration(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack().first_mut().unwrap().vcustom_mut().custom.as_any_mut();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let (Some(d1), Some(d2)) = (v1custom.downcast_ref::<DurationCustom>(), v2custom.downcast_mut::<DurationCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(dsum) = d1.duration.checked_add(d2.duration) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("DURATION OVERFLOW", w)
  };
  let _ = std::mem::replace(&mut d2.duration, dsum);
  state.pool.add_val(v1);
  state.current().stack.push(v2);
  state
}

pub fn cog_sub_duration(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack().first_mut().unwrap().vcustom_mut().custom.as_any_mut();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let (Some(d1), Some(d2)) = (v1custom.downcast_ref::<DurationCustom>(), v2custom.downcast_mut::<DurationCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(dsum) = d1.duration.checked_sub(d2.duration) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("DURATION UNDERFLOW", w)
  };
  let _ = std::mem::replace(&mut d2.duration, dsum);
  state.pool.add_val(v1);
  state.current().stack.push(v2);
  state
}

pub fn cog_duration_sum_overload(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 4 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v4 = state.current().stack.pop().unwrap();
  let v3 = state.current().stack.pop().unwrap();
  let v2 = state.current().stack.pop().unwrap();
  let v1 = state.current().stack.last().unwrap();
  if let Some(Value::Custom(vcustom1)) = v1.value_stack_ref().first() {
    if let Some(Value::Custom(vcustom2)) = v2.value_stack_ref().first() {
      let v1isduration = vcustom1.custom.as_any().downcast_ref::<DurationCustom>().is_some();
      let v2isduration = vcustom2.custom.as_any().downcast_ref::<DurationCustom>().is_some();
      if v1isduration && v2isduration {
        state.current().stack.push(v4);
        state.pool.add_val(v3);
        return state
      }
    }
  }
  state.current().stack.push(v3);
  state.pool.add_val(v4);
  state
}

pub fn cog_later(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let mut v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack().first_mut().unwrap().vcustom_mut().custom.as_any_mut();
  let (Some(i), Some(d)) = (v1custom.downcast_mut::<InstantCustom>(), v2custom.downcast_ref::<DurationCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(i_new) = i.instant.checked_add(d.duration) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("INSTANT OVERFLOW", w)
  };
  let _ = std::mem::replace(&mut i.instant, i_new);
  state.pool.add_val(v2);
  state.current().stack.push(v1);
  state
}

pub fn cog_earlier(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let mut v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack().first_mut().unwrap().vcustom_mut().custom.as_any_mut();
  let (Some(i), Some(d)) = (v1custom.downcast_mut::<InstantCustom>(), v2custom.downcast_ref::<DurationCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(i_new) = i.instant.checked_sub(d.duration) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("INSTANT UNDERFLOW", w)
  };
  let _ = std::mem::replace(&mut i.instant, i_new);
  state.pool.add_val(v2);
  state.current().stack.push(v1);
  state
}

pub fn cog_duration(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<InstantCustom>(), v2custom.downcast_ref::<InstantCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(d) = i2.instant.checked_duration_since(i1.instant) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("NEGATIVE DURATION", w)
  };
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  let vcustom = get_duration_custom(&mut state.pool, d);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_elapsed(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let Some(instant) = custom.downcast_ref::<InstantCustom>() else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let vcustom = get_duration_custom(&mut state.pool, instant.instant.elapsed());
  state.pool.add_val(v);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_now(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vcustom = get_instant_custom(&mut state.pool, Instant::now());
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_duration_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let vword = if custom.downcast_ref::<DurationCustom>().is_some() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_instant_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let vword = if custom.downcast_ref::<InstantCustom>().is_some() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_later_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<InstantCustom>(), v2custom.downcast_ref::<InstantCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let vword = if i2.instant.checked_duration_since(i1.instant).is_some() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.current().stack.push(v1);
  state.current().stack.push(v2);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_earlier_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<InstantCustom>(), v2custom.downcast_ref::<InstantCustom>()) else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let vword = if i2.instant.checked_duration_since(i1.instant).is_none() {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.current().stack.push(v1);
  state.current().stack.push(v2);
  state.push_quoted(Value::Word(vword));
  state
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  add_word!(state, lib, "nanos", cog_nanos);
  add_word!(state, lib, "micros", cog_micros);
  add_word!(state, lib, "millis", cog_millis);
  add_word!(state, lib, "secs", cog_secs);
  add_word!(state, lib, "as-nanos", cog_as_nanos);
  add_word!(state, lib, "as-micros", cog_as_micros);
  add_word!(state, lib, "as-millis", cog_as_millis);
  add_word!(state, lib, "as-secs", cog_as_secs);
  add_word!(state, lib, "later", cog_later);
  add_word!(state, lib, "earlier", cog_earlier);
  add_word!(state, lib, "duration", cog_duration);
  add_word!(state, lib, "elapsed", cog_elapsed);
  add_word!(state, lib, "now", cog_now);
  add_word!(state, lib, "duration?", cog_duration_questionmark);
  add_word!(state, lib, "instant?", cog_instant_questionmark);
  add_word!(state, lib, "later?", cog_later_questionmark);
  add_word!(state, lib, "earlier?", cog_earlier_questionmark);
  overload_word!(state, lib, "+", cog_duration_sum_overload, cog_add_duration);
  overload_word!(state, lib, "-", cog_duration_sum_overload, cog_sub_duration);
}
