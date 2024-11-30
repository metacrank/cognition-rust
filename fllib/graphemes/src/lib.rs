use cognition::*;
use unicode_segmentation::UnicodeSegmentation;

pub fn cog_gunconcat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let mut unconcatlen: usize = 0;
  for val in v.value_stack_ref().iter() {
    let Value::Word(word) = val else {
      stack.push(v);
      return state.eval_error("BAD ARGUMENT TYPE", w)
    };
    unconcatlen += word.str_word.len()
  }
  let mut newstack = state.pool.get_vstack(unconcatlen);
  for val in v.value_stack_ref().iter() {
    let string = &val.vword_ref().str_word;
    for g in string.graphemes(true) {
      let mut vword = state.pool.get_vword(1);
      vword.str_word.push_str(g);
      newstack.container.stack.push(Value::Word(vword))
    }
  }
  state.pool.add_val(v);
  state.current().stack.push(Value::Stack(newstack));
  state
}

pub fn cog_gcut(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(vint) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(mut vstr) = stack.pop() else {
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w);
  };
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let Some(math) = state.get_math() else {
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w);
  };
  let string = &mut vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let mut iter = string.grapheme_indices(true);
  let result = math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word);
  state.set_math(math);
  match result {
    Ok(mut i) => if i < 0 || i.abs() as usize > string.len() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      loop {
        if i == 0 { break }
        if iter.next().is_none() {
          state.current().stack.push(vstr);
          state.current().stack.push(vint);
          return state.eval_error("OUT OF BOUNDS", w)
        }
        i -= 1
      }
    },
    Err(e) => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w);
    },
  };
  let new_word = match iter.next() {
    Some((i, _)) => {
      let mut new_word = state.pool.get_vword(string.len() - i);
      new_word.str_word.push_str(&string[i..]);
      string.truncate(i);
      new_word
    },
    None => state.pool.get_vword(0),
  };
  state.current().stack.push(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_gsubstr(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let vint2 = stack.pop().unwrap();
  let vint1 = stack.pop().unwrap();
  let mut vstr = stack.pop().unwrap();
  if vint1.value_stack_ref().len() != 1 || vint1.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vstr);
    stack.push(vint1);
    stack.push(vint2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !vint1.value_stack_ref().first().unwrap().is_word() || !vint2.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vstr);
    stack.push(vint1);
    stack.push(vint2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Some(math) = state.get_math() else {
    state.current().stack.push(vstr);
    state.current().stack.push(vint1);
    state.current().stack.push(vint2);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let string = &mut vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let int1 = match math.math().stoi(&vint1.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i >= 0 { i as usize } else {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    },
    Err(e) => {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error(e, w)
    },
  };
  let int2 = match math.math().stoi(&vint2.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i >= 0 { i as usize } else {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    },
    Err(e) => {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error(e, w)
    },
  };
  state.set_math(math);
  if int1 >= int2 {
    let glen = string.graphemes(true).count();
    if int1 <= glen && int2 <= glen {
      string.clear();
      state.current().stack.push(vstr);
      state.pool.add_val(vint1);
      state.pool.add_val(vint2);
      return state
    } else {
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  let mut iter = string.grapheme_indices(true);
  for _ in 0..int1 {
    if iter.next().is_none() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  let i1 = match iter.next() {
    Some((i, _)) => i,
    None => string.len(),
  };
  for _ in int1..(int2-1) {
    if iter.next().is_none() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  let i2 = match iter.next() {
    Some((i, _)) => i,
    None => string.len(),
  };
  state.pool.add_val(vint1);
  state.pool.add_val(vint2);
  let mut new_word = state.pool.get_vword(i2 - i1);
  new_word.str_word.push_str(&string[i1..i2]);
  state.pool.add_val(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_glen(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = v_stack.first().unwrap();
  if !word_v.is_word() { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let length = word_v.vword_ref().str_word.graphemes(true).count();
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  if math.math().base() == 1 && length != 0 {
    return state.with_math(math).eval_error("MATH BASE ONE", w)
  }
  if length > isize::MAX as usize { return state.with_math(math).eval_error("OUT OF BOUNDS", w) }
  match math.math().itos(length as isize, &mut state) {
    Ok(s) => {
      state.set_math(math);
      let mut v = state.pool.get_vword(s.len());
      v.str_word.push_str(&s);
      state.pool.add_string(s);
      state.push_quoted(Value::Word(v));
      state
    },
    Err(e) => return state.with_math(math).eval_error(e, w),
  }
}

pub fn cog_gat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut vint = stack.pop().unwrap();
  let vstr = stack.pop().unwrap();
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Some(math) = state.get_math() else {
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let vstr_stack = vstr.value_stack_ref();
  let string = &vstr_stack.first().unwrap().vword_ref().str_word;
  let result = math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word);
  state.set_math(math);
  let int = match result {
    Ok(i) => if i < 0 || i.abs() as usize >= string.len() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      i as usize
    },
    Err(e) => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w)
    },
  };
  if !string.is_char_boundary(int) {
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("INVALID CHAR BOUNDARY", w)
  }
  let s = &mut vint.value_stack().first_mut().unwrap().vword_mut().str_word;
  s.clear();
  s.push_str(string[int..].graphemes(true).next().unwrap());
  state.current().stack.push(vstr);
  state.current().stack.push(vint);
  state
}

pub fn cog_gnth(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut vint = stack.pop().unwrap();
  let vstr = stack.pop().unwrap();
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Some(math) = state.get_math() else {
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let vstr_stack = vstr.value_stack_ref();
  let string = &vstr_stack.first().unwrap().vword_ref().str_word;
  let result = math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word);
  state.set_math(math);
  let int = match result {
    Ok(i) => if i < 0 || i.abs() as usize >= string.len() {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      i as usize
    },
    Err(e) => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w)
    },
  };
  let mut iter = string.graphemes(true);
  for _ in 0..int { iter.next(); }
  match iter.next() {
    Some(g) => {
      let s = &mut vint.value_stack().first_mut().unwrap().vword_mut().str_word;
      s.clear();
      s.push_str(g);
    },
    None => {
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  state.current().stack.push(vstr);
  state.current().stack.push(vint);
  state
}

pub fn cog_ginsert(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(vint) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(mut vstr) = stack.pop() else {
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w);
  };
  let Some(vsrc) = stack.pop() else {
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
    stack.push(vsrc);
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  if !vint.value_stack_ref().first().unwrap().is_word() || !vstr.value_stack_ref().first().unwrap().is_word() {
    stack.push(vsrc);
    stack.push(vstr);
    stack.push(vint);
    return state.eval_error("BAD ARGUMENT TYPE", w);
  }
  let Some(math) = state.get_math() else {
    state.current().stack.push(vsrc);
    state.current().stack.push(vstr);
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w);
  };
  let string = &vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let source = &vsrc.value_stack_ref().first().unwrap().vword_ref().str_word;
  let mut iter = source.grapheme_indices(true);
  let result = math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word);
  state.set_math(math);
  match result {
    Ok(mut i) => if i < 0 || i.abs() as usize > source.len() {
      state.current().stack.push(vsrc);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      loop {
        if i == 0 { break }
        if iter.next().is_none() {
          state.current().stack.push(vsrc);
          state.current().stack.push(vstr);
          state.current().stack.push(vint);
          return state.eval_error("OUT OF BOUNDS", w)
        }
        i -= 1
      }
    },
    Err(e) => {
      state.current().stack.push(vsrc);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w);
    },
  };
  let idx = match iter.next() {
    Some((i, _)) => i,
    None => source.len()
  };
  let mut new_word = state.pool.get_vword(string.len() + source.len());
  new_word.str_word.push_str(&source[..idx]);
  new_word.str_word.push_str(&string);
  new_word.str_word.push_str(&source[idx..]);

  state.pool.add_val(vint);
  state.pool.add_val(vstr);
  state.pool.add_val(vsrc);

  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_greverse(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let Value::Word(vw) = v.value_stack_ref().first().unwrap() else {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let mut new_vword = state.pool.get_vword(vw.str_word.len());
  for g in vw.str_word.graphemes(true).rev() {
    new_vword.str_word.push_str(g);
  }
  state.pool.add_val(v.value_stack().pop().unwrap());
  v.value_stack().push(Value::Word(new_vword));
  state.current().stack.push(v);
  state
}

pub fn cog_gtoi(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(mut v) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  if v.value_stack_ref().len() != 1 {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let char_v = v.value_stack().first_mut().unwrap();
  let Value::Word(vword_char) = char_v else {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let mut graphemes = vword_char.str_word.graphemes(true);
  let g = graphemes.next();
  if g.is_none() || graphemes.next().is_some() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(math) = state.get_math() else {
    state.current().stack.push(v);
    return state.eval_error("MATH BASE ZERO", w)
  };
  let mut agg: isize = 0;
  for c in g.unwrap().chars().rev() {
    agg = agg * 0x100000000 + c as isize;
  }
  match math.math().itos(agg, &mut state) {
    Ok(mut s) => {
      state.set_math(math);
      std::mem::swap(&mut vword_char.str_word, &mut s);
      state.pool.add_string(s);
      state.current().stack.push(v);
      state
    },
    Err(e) => {
      state.set_math(math);
      state.current().stack.push(v);
      state.eval_error(e, w)
    }
  }
}

#[no_mangle]
pub extern fn add_words(state: &mut CognitionState, lib: &Library) {
  ensure_foreign_library!(state, lib);
  add_word!(state, lib, "gunconcat", cog_gunconcat);
  add_word!(state, lib, "gcut", cog_gcut);
  add_word!(state, lib, "gsubstr", cog_gsubstr);
  add_word!(state, lib, "glen", cog_glen);
  add_word!(state, lib, "gat", cog_gat);
  add_word!(state, lib, "gnth", cog_gnth);
  add_word!(state, lib, "ginsert", cog_ginsert);
  add_word!(state, lib, "greverse", cog_greverse);
  add_word!(state, lib, "gtoi", cog_gtoi);
}
