use crate::*;

pub fn cog_crank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let base = get_int!(state, w);
  let cur = state.current();
  if cur.cranks.is_none() {
    state.current().cranks = Some(state.pool.get_cranks(1));
  }
  let modulo = if base > 1 { 1 } else { 0 };
  if let Some(crank) = state.current().cranks.as_mut().unwrap().first_mut() {
    crank.modulo = modulo;
    crank.base = base;
  } else {
    state.current().cranks.as_mut().unwrap().push(Crank { modulo, base });
  }
  state
}

pub fn cog_metacrank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let (meta, base) = get_2_ints!(state, w);
  let meta = meta as usize;
  if state.current().cranks.is_none() {
    state.current().cranks = Some(state.pool.get_cranks(meta));
  }
  let cranks = state.current().cranks.as_mut().unwrap();
  for _ in cranks.len()..meta {
    cranks.push(Crank { modulo: 0, base: 0 });
  }
  let modulo = if base > 1 { 1 } else { 0 };
  if let Some(crank) = cranks.get_mut(meta) {
    crank.modulo = modulo;
    crank.base = base;
  } else {
    state.current().cranks.as_mut().unwrap().push(Crank { modulo, base });
  }
  state
}

pub fn cog_halt(mut state: CognitionState, _: Option<&Value>) -> CognitionState {
  if let Some(cranks) = &mut state.current().cranks { cranks.clear() }
  state
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "crank", cog_crank);
  add_word!(state, "metacrank", cog_metacrank);
  add_word!(state, "halt", cog_halt);
}
