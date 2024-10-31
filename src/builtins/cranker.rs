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

macro_rules! cog_crank_val {
  ($state:ident,$w:ident,$letpat:pat,$valexpr:expr) => {{
    let Some(math) = $state.current().math.take() else { return $state.eval_error("MATH BASE ZERO", $w) };
    let val = if let Some(ref cranks) = $state.current().cranks {
      if let $letpat = cranks.get(0) { $valexpr } else { 0 }
    } else { 0 };
    let s = math.itos(val as isize, &mut $state);
    $state.current().math = Some(math);
    match s {
      Ok(s) => {
        let mut vword = $state.pool.get_vword(s.len());
        vword.str_word.push_str(&s);
        $state.pool.add_string(s);
        $state.current().stack.push(Value::Word(vword));
        $state
      },
      Err(e) => $state.eval_error(e, $w)
    }
  }};
}
macro_rules! cog_metacrank_val {
  ($state:ident,$w:ident,$letpat:pat,$valexpr:expr) => {{
    let idx = get_int!($state, $w, usize, ACTIVE);
    let cur = $state.current();
    let math = cur.math.take().unwrap();
    let base = if let Some(ref cranks) = cur.cranks {
      if let $letpat = cranks.get(idx) { $valexpr } else { 0 }
    } else { 0 };
    let s = math.itos(base as isize, &mut $state);
    $state.current().math = Some(math);
    match s {
      Ok(mut s) => {
        let vword_stack = $state.current().stack.last_mut().unwrap().value_stack();
        let vword = vword_stack.first_mut().unwrap().vword_mut();
        std::mem::swap(&mut s, &mut vword.str_word);
        $state.pool.add_string(s);
        $state
      },
      Err(e) => $state.eval_error(e, $w),
    }
  }};
}

pub fn cog_crankbase(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  cog_crank_val!(state, w, Some(crank), crank.base)
}

pub fn cog_modcrank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  cog_crank_val!(state, w, Some(crank), crank.modulo)
}

pub fn cog_metacrankbase(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  cog_metacrank_val!(state, w, Some(crank), crank.base)
}

pub fn cog_metamodcrank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  cog_metacrank_val!(state, w, Some(crank), crank.modulo)
}

pub fn add_words(state: &mut CognitionState) {
  add_word!(state, "crank", cog_crank);
  add_word!(state, "metacrank", cog_metacrank);
  add_word!(state, "halt", cog_halt);
  add_word!(state, "crankbase", cog_crankbase);
  add_word!(state, "modcrank", cog_modcrank);
  add_word!(state, "metacrankbase", cog_metacrankbase);
  add_word!(state, "metamodcrank", cog_metamodcrank);
}
