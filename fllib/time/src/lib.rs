pub mod custom;

pub use crate::custom::*;
use cognition::*;

macro_rules! duration_from {
  ($state:ident,$w:ident,$from:expr) => {
    let i = get_int!($state, $w, isize, ACTIVE, "DURATION OVERFLOW");
    let neg = i < 0;
    let i = if neg { -i } else { i } as usize;
    if i > u64::MAX as usize { return $state.eval_error("DURATION OVERFLOW", $w) }
    let v = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v);
    let vcustom = get_duration_custom(&mut $state.pool, $from(i as u64), neg);
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
    let i = if duration_custom.neg { (i as isize) * -1 } else { i as isize };
    let mut s = match math.math().itos(i, &mut $state) {
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

macro_rules! add_durations {
  ($state:ident,$w:ident,$v1:ident,$v2:ident,$d1:ident,$d2:ident) => {
    if $d1.neg != $d2.neg {
      let dsum = match $d2.duration.checked_sub($d1.duration) {
        Some(sum) => sum,
        None => {
          $d2.neg = !$d2.neg;
          match $d1.duration.checked_sub($d2.duration) {
            Some(sum) => sum,
            None => Duration::ZERO
          }
        }
      };
      $d2.duration = dsum;
    } else {
      let Some(dsum) = $d1.duration.checked_add($d2.duration) else {
        $state.current().stack.push($v1);
        $state.current().stack.push($v2);
        return $state.eval_error("DURATION OVERFLOW", $w)
      };
      $d2.duration = dsum;
    }
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
  println!("about to add durations");
  add_durations!(state, w, v1, v2, d1, d2);
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
  d2.neg = !d2.neg;
  add_durations!(state, w, v1, v2, d1, d2);
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
        println!("duration add");
        state.pool.add_val(v3);
        state.current().stack.push(v2);
        let wd = WordDef::from(v4);
        return state.evalstack(wd, w, false)
      }
    }
  }
  println!("non-duration add");
  state.pool.add_val(v4);
  state.current().stack.push(v2);
  let wd = WordDef::from(v3);
  state.evalstack(wd, w, false)
}

fn add_duration(mut state: CognitionState, w: Option<&Value>, add: bool) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let mut v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack().first_mut().unwrap().vcustom_mut().custom.as_any_mut();
  let overflow = 'fail: {
    if let (Some(i), Some(d)) = (v1custom.downcast_mut::<InstantCustom>(), v2custom.downcast_ref::<DurationCustom>()) {
      let i_new = if add {
        let Some(i) = i.instant.checked_add(d.duration) else { break 'fail !d.neg }; i
      } else {
        let Some(i) = i.instant.checked_sub(d.duration) else { break 'fail d.neg }; i
      };
      i.instant = i_new;
    } else if let (Some(i), Some(d)) = (v1custom.downcast_mut::<SystemTimeCustom>(), v2custom.downcast_ref::<DurationCustom>()) {
      let t_new = if add {
        let Some(t) = i.time.checked_add(d.duration) else { break 'fail !d.neg }; t
      } else {
        let Some(t) = i.time.checked_sub(d.duration) else { break 'fail d.neg }; t
      };
      i.time = t_new;
    } else {
      state.current().stack.push(v1);
      state.current().stack.push(v2);
      return state.eval_error("BAD ARGUMENT TYPE", w)
    }
    state.pool.add_val(v2);
    state.current().stack.push(v1);
    return state
  };
  state.current().stack.push(v1);
  state.current().stack.push(v2);
  if overflow {
    state.eval_error("INSTANT OVERFLOW", w)
  } else {
    state.eval_error("INSTANT UNDERFLOW", w)
  }
}

pub fn cog_later(state: CognitionState, w: Option<&Value>) -> CognitionState {
  add_duration(state, w, true)
}
pub fn cog_earlier(state: CognitionState, w: Option<&Value>) -> CognitionState {
  add_duration(state, w, false)
}

pub fn cog_duration(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let mut neg = false;
  let duration = if let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<InstantCustom>(), v2custom.downcast_ref::<InstantCustom>()) {
    match i2.instant.checked_duration_since(i1.instant) {
      Some(d) => d,
      None => {
        neg = true;
        match i1.instant.checked_duration_since(i2.instant) {
          Some(d) => d,
          None => Duration::ZERO
        }
      }
    }
  } else if let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<SystemTimeCustom>(), v2custom.downcast_ref::<SystemTimeCustom>()) {
    match i2.time.duration_since(i1.time) {
      Ok(d) => d,
      Err(e) => e.duration()
    }
  } else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let vcustom = get_duration_custom(&mut state.pool, duration, neg);
  state.push_quoted(Value::Custom(vcustom));
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  state
}

pub fn cog_elapsed(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let vcustom = if let Some(instant) = custom.downcast_ref::<InstantCustom>() {
    get_duration_custom(&mut state.pool, instant.instant.elapsed(), false)
  } else if let Some(system_time) = custom.downcast_ref::<SystemTimeCustom>() {
    match system_time.time.elapsed() {
      Ok(duration) => get_duration_custom(&mut state.pool, duration, false),
      Err(e) => get_duration_custom(&mut state.pool, e.duration(), true)
    }
  } else {
    state.current().stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  state.pool.add_val(v);
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_now(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vcustom = get_instant_custom(&mut state.pool, Instant::now());
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_system_time(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  let vcustom = get_system_time_custom(&mut state.pool, SystemTime::now());
  state.push_quoted(Value::Custom(vcustom));
  state
}

pub fn cog_duration_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let vword = if custom.is::<DurationCustom>() {
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
  let vword = if custom.is::<InstantCustom>() {
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

pub fn cog_system_time_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let v = get_custom!(state, w);
  let custom = v.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let vword = if custom.is::<SystemTimeCustom>() {
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

fn is_later(mut state: CognitionState, w: Option<&Value>, result: &mut bool) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_custom!(state, w);
  let v1 = get_custom!(state, w, { state.current().stack.push(v2) });
  let v2custom = v2.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  let v1custom = v1.value_stack_ref().first().unwrap().vcustom_ref().custom.as_any();
  if let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<InstantCustom>(), v2custom.downcast_ref::<InstantCustom>()) {
    *result = i2.instant.checked_duration_since(i1.instant).is_some()
  } else if let (Some(i1), Some(i2)) = (v1custom.downcast_ref::<SystemTimeCustom>(), v2custom.downcast_ref::<SystemTimeCustom>()) {
    *result = i2.time.duration_since(i1.time).is_ok()
  } else {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  state.current().stack.push(v1);
  state.current().stack.push(v2);
  state
}

pub fn cog_later_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut result = false;
  state = is_later(state, w, &mut result);
  let vword = if result {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  } else {
    state.pool.get_vword(0)
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_earlier_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let mut result = false;
  state = is_later(state, w, &mut result);
  let vword = if result {
    state.pool.get_vword(0)
  } else {
    let mut vword = state.pool.get_vword(1);
    vword.str_word.push('t');
    vword
  };
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_clear_time_pools(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  clear_pools(&mut state.pool);
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
  add_word!(state, lib, "system-time", cog_system_time);
  add_word!(state, lib, "duration?", cog_duration_questionmark);
  add_word!(state, lib, "instant?", cog_instant_questionmark);
  add_word!(state, lib, "system-time?", cog_system_time_questionmark);
  add_word!(state, lib, "later?", cog_later_questionmark);
  add_word!(state, lib, "earlier?", cog_earlier_questionmark);
  add_word!(state, lib, "clear-time-pools", cog_clear_time_pools);

  overload_word!(state, lib, "+", cog_duration_sum_overload, cog_add_duration);
  overload_word!(state, lib, "-", cog_duration_sum_overload, cog_sub_duration);

  state.add_const_custom("ZERO_DURATION", Box::new(DurationCustom{ duration: Duration::ZERO, neg: false }));
  state.add_const_custom("MAX_DURATION", Box::new(DurationCustom{ duration: Duration::MAX, neg: false }));
  state.add_const_custom("MIN_DURATION", Box::new(DurationCustom{ duration: Duration::MAX, neg: true }));
  state.add_const_custom("NANOSECOND", Box::new(DurationCustom{ duration: Duration::from_nanos(1), neg: false }));
  state.add_const_custom("MICROSECOND", Box::new(DurationCustom{ duration: Duration::from_micros(1), neg: false }));
  state.add_const_custom("MILLISECOND", Box::new(DurationCustom{ duration: Duration::from_millis(1), neg: false }));
  state.add_const_custom("SECOND", Box::new(DurationCustom{ duration: Duration::from_secs(1), neg: false }));
  state.add_const_custom("UNIX_EPOCH", Box::new(SystemTimeCustom{ time: SystemTime::UNIX_EPOCH }));
}
