use crate::*;

pub fn cog_crank(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let base = get_unsigned!(state, w);
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
  let (meta, base) = get_2_unsigned!(state, w, isize, ACTIVE);
  if meta < 0 || base < 0 || base > i32::MAX as isize { return state.eval_error("OUT OF BOUNDS", w) }
  let meta = meta as usize;
  let base = base as i32;
  for _ in 0..2 { let v = state.current().stack.pop().unwrap(); state.pool.add_val(v) }
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
    let val = if let Some(ref cranks) = $state.current().cranks {
      if let $letpat = cranks.get(0) { $valexpr } else { 0 }
    } else { 0 };
    let Some(math) = $state.get_math() else { return $state.eval_error("MATH BASE ZERO", $w) };
    match math.math().itos(val as isize, &mut $state) {
      Ok(s) => {
        $state.set_math(math);
        let mut vword = $state.pool.get_vword(s.len());
        vword.str_word.push_str(&s);
        $state.pool.add_string(s);
        $state.push_quoted(Value::Word(vword));
        $state
      },
      Err(e) => $state.with_math(math).eval_error(e, $w)
    }
  }};
}
macro_rules! cog_metacrank_val {
  ($state:ident,$w:ident,$letpat:pat,$valexpr:expr) => {{
    let idx = get_unsigned!($state, $w, isize, ACTIVE) as usize;
    let base = if let Some(ref cranks) = $state.current().cranks {
      if let $letpat = cranks.get(idx) { $valexpr } else { 0 }
    } else { 0 };
    let math = $state.get_math().unwrap();
    let s = math.math().itos(base as isize, &mut $state);
    match s {
      Ok(mut s) => {
        $state.set_math(math);
        let vword_stack = $state.current().stack.last_mut().unwrap().value_stack();
        let vword = vword_stack.first_mut().unwrap().vword_mut();
        std::mem::swap(&mut s, &mut vword.str_word);
        $state.pool.add_string(s);
        $state
      },
      Err(e) => $state.with_math(math).eval_error(e, $w),
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

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "crank", cog_crank);
  add_builtin!(state, "metacrank", cog_metacrank);
  add_builtin!(state, "halt", cog_halt);
  add_builtin!(state, "crankbase", cog_crankbase);
  add_builtin!(state, "modcrank", cog_modcrank);
  add_builtin!(state, "metacrankbase", cog_metacrankbase);
  add_builtin!(state, "metamodcrank", cog_metamodcrank);
}
