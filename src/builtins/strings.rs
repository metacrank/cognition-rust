use crate::*;

pub fn cog_concat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v2 = stack.last().unwrap();
  let v1 = stack.get(stack.len() - 2).unwrap();
  if v1.value_stack_ref().len() == 0 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let mut concatlen: usize = 0;
  for v in v1.value_stack_ref().iter() {
    let Value::Word(vw) = v else { return state.eval_error("BAD ARGUMENT TYPE", w) };
    concatlen += vw.str_word.len()
  }
  for v in v2.value_stack_ref().iter() {
    let Value::Word(vw) = v else { return state.eval_error("BAD ARGUMENT TYPE", w) };
    concatlen += vw.str_word.len()
  }
  let mut new_word = state.pool.get_vword(concatlen);
  let mut iterate = |v: &Value| for val in v.value_stack_ref().iter() {
    new_word.str_word.push_str(&val.vword_ref().str_word)
  };
  let v2 = state.current().stack.pop().unwrap();
  let v1 = state.current().stack.pop().unwrap();
  iterate(&v1);
  iterate(&v2);
  state.pool.add_val(v2);
  state.pool.add_val(v1);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_unconcat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
    for c in string.chars() {
      let mut vword = state.pool.get_vword(1);
      vword.str_word.push(c);
      newstack.container.stack.push(Value::Word(vword))
    }
  }
  state.pool.add_val(v);
  state.current().stack.push(Value::Stack(newstack));
  state
}

pub fn cog_cut(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  let Some(vint) = stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let Some(mut vstr) = stack.pop() else {
    stack.push(vint);
    return state.eval_error("TOO FEW ARGUMENTS", w)
  };
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
  let string = &mut vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let int = match math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize > string.len() {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      state.set_math(math);
      i as usize
    },
    Err(e) => {
      state.set_math(math);
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
  state.pool.add_val(vint);
  let mut new_word = state.pool.get_vword(string.len() - int);
  new_word.str_word.push_str(&string[int..]);
  string.truncate(int);

  state.current().stack.push(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_ccut(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
    state.current().stack.push(vint);
    return state.eval_error("MATH BASE ZERO", w);
  };
  let string = &mut vstr.value_stack().first_mut().unwrap().vword_mut().str_word;
  let mut iter = string.char_indices();
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

pub fn cog_substr(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
    Ok(i) => if i < 0 || i.abs() as usize > string.len() {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      i as usize
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
    Ok(i) => if i < 0 || i.abs() as usize > string.len() {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      state.set_math(math);
      i as usize
    },
    Err(e) => {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint1);
      state.current().stack.push(vint2);
      return state.eval_error(e, w)
    },
  };
  if !string.is_char_boundary(int1) || !string.is_char_boundary(int1) {
    state.current().stack.push(vstr);
    state.current().stack.push(vint1);
    state.current().stack.push(vint2);
    return state.eval_error("INVALID CHAR BOUNDARY", w)
  }
  state.pool.add_val(vint1);
  state.pool.add_val(vint2);
  if int1 >= int2 {
    string.clear();
    state.current().stack.push(vstr);
    return state
  }
  let mut new_word = state.pool.get_vword(int2 - int1);
  new_word.str_word.push_str(&string[int1..int2]);
  state.pool.add_val(vstr);
  state.push_quoted(Value::Word(new_word));
  state
}

pub fn cog_csubstr(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
    let glen = string.chars().count();
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
  let mut iter = string.char_indices();
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

pub fn cog_len(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = v_stack.first().unwrap();
  if !word_v.is_word() { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let length = word_v.vword_ref().str_word.len();
  let Some(math) = state.get_math() else { return state.eval_error("MATH BASE ZERO", w) };
  if math.math().base() == 0 { return state.with_math(math).eval_error("MATH BASE ZERO", w) }
  if length > isize::MAX as usize { return state.eval_error("OUT OF BOUNDS", w) }
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

pub fn cog_clen(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.last() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let v_stack = v.value_stack_ref();
  if v_stack.len() != 1 { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let word_v = v_stack.first().unwrap();
  if !word_v.is_word() { return state.eval_error("BAD ARGUMENT TYPE", w) }
  let length = word_v.vword_ref().str_word.chars().count();
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

pub fn cog_cat(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  let int = match math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize >= string.len() {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      state.set_math(math);
      i as usize
    },
    Err(e) => {
      state.set_math(math);
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
  s.push(string[int..].chars().next().unwrap().clone());
  state.current().stack.push(vint);
  state
}

pub fn cog_nth(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let stack = &mut state.current().stack;
  if stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let mut vint = stack.pop().unwrap();
  let vstr = stack.pop().unwrap();
  if vint.value_stack_ref().len() != 1 || vstr.value_stack_ref().len() != 1 {
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
  let int = match math.math().stoi(&vint.value_stack_ref().first().unwrap().vword_ref().str_word) {
    Ok(i) => if i < 0 || i.abs() as usize >= string.len() {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    } else {
      state.set_math(math);
      i as usize
    },
    Err(e) => {
      state.set_math(math);
      state.current().stack.push(vstr);
      state.current().stack.push(vint);
      return state.eval_error(e, w)
    },
  };
  let mut iter = string.chars();
  for _ in 0..int { iter.next(); }
  match iter.next() {
    Some(c) => {
      let s = &mut vint.value_stack().first_mut().unwrap().vword_mut().str_word;
      s.clear();
      s.push(c);
    },
    None => {
      state.current().stack.push(vint);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  state.current().stack.push(vint);
  state
}

pub fn cog_replace(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 4 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let (i, j) = get_2_unsigned!(state, w, isize, ACTIVE);
  let i = i as usize;
  let j = j as usize;
  if i > j { return state.eval_error("OUT OF BOUNDS", w) }
  let j_val = state.current().stack.pop().unwrap();
  let i_val = state.current().stack.pop().unwrap();
  let (mut v1, v2) = get_2_words!(state, w, {
    state.current().stack.push(i_val);
    state.current().stack.push(j_val);
  });
  let stack = &mut state.current().stack;
  let str1 = &mut v1.value_stack().first_mut().unwrap().vword_mut().str_word;
  let str2 = &v2.value_stack_ref().first().unwrap().vword_ref().str_word;
  if !str1.is_char_boundary(i) || !str1.is_char_boundary(j) {
    stack.push(v1);
    stack.push(v2);
    stack.push(i_val);
    stack.push(j_val);
    return state.eval_error("OUT OF BOUNDS", w)
  }
  str1.replace_range(i..j, str2);
  state.pool.add_val(i_val);
  state.pool.add_val(j_val);
  state.pool.add_val(v2);
  state.current().stack.push(v1);
  state
}

pub fn cog_creplace(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 4 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let (i, j) = get_2_unsigned!(state, w, isize, ACTIVE);
  let i = i as usize;
  let j = j as usize;
  if i > j { return state.eval_error("OUT OF BOUNDS", w) }
  let j_val = state.current().stack.pop().unwrap();
  let i_val = state.current().stack.pop().unwrap();
  let (mut v1, v2) = get_2_words!(state, w, {
    state.current().stack.push(i_val);
    state.current().stack.push(j_val);
  });
  let stack = &mut state.current().stack;
  let str1 = &mut v1.value_stack().first_mut().unwrap().vword_mut().str_word;
  let str2 = &v2.value_stack_ref().first().unwrap().vword_ref().str_word;
  let mut iter = str1.char_indices();
  for _ in 0..i {
    if iter.next().is_none() {
      stack.push(v1);
      stack.push(v2);
      stack.push(i_val);
      stack.push(j_val);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  let int1 = match iter.next() {
    Some((i, _)) => i,
    None => str1.len(),
  };
  for _ in i..(j-1) {
    if iter.next().is_none() {
      stack.push(v1);
      stack.push(v2);
      stack.push(i_val);
      stack.push(j_val);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  let int2 = match iter.next() {
    Some((i, _)) => i,
    None => str1.len(),
  };
  str1.replace_range(int1..int2, str2);
  state.pool.add_val(i_val);
  state.pool.add_val(j_val);
  state.pool.add_val(v2);
  state.current().stack.push(v1);
  state
}

pub fn cog_slice(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let (i, j) = get_2_unsigned!(state, w, isize, ACTIVE);
  let i = i as usize;
  let j = j as usize;
  let j_val = state.current().stack.pop().unwrap();
  let i_val = state.current().stack.pop().unwrap();
  let mut v1 = get_word!(state, w, {
    state.current().stack.push(i_val);
    state.current().stack.push(j_val);
  });
  let stack = &mut state.current().stack;
  let string = &mut v1.value_stack().first_mut().unwrap().vword_mut().str_word;
  if !string.is_char_boundary(i) || !string.is_char_boundary(j) {
    stack.push(v1);
    stack.push(i_val);
    stack.push(j_val);
    return state.eval_error("OUT OF BOUNDS", w)
  }
  if i >= j {
    let vw = state.pool.get_vword(0);
    state.current().stack.push(v1);
    state.push_quoted(Value::Word(vw));
    state.pool.add_val(i_val);
    state.pool.add_val(j_val);
    return state
  }
  let mut vw = state.pool.get_vword(j - i);
  vw.str_word.push_str(&string[i..j]);
  state.pool.add_val(i_val);
  state.pool.add_val(j_val);
  state.current().stack.push(v1);
  state.push_quoted(Value::Word(vw));
  state
}

pub fn cog_cslice(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current_ref().stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let (i, j) = get_2_unsigned!(state, w, isize, ACTIVE);
  let i = i as usize;
  let j = j as usize;
  let stack = &mut state.current().stack;
  let j_val = stack.pop().unwrap();
  let i_val = stack.pop().unwrap();
  let mut v1 = get_word!(state, w, {
    state.current().stack.push(i_val);
    state.current().stack.push(j_val);
  });
  let stack = &mut state.current().stack;
  let string = &mut v1.value_stack().first_mut().unwrap().vword_mut().str_word;
  let mut iter = string.char_indices();
  for _ in 0..i {
    if iter.next().is_none() {
      stack.push(v1);
      stack.push(i_val);
      stack.push(j_val);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  let int1 = match iter.next() {
    Some((i, _)) => i,
    None => string.len(),
  };
  for _ in i..(j-1) {
    if iter.next().is_none() {
      stack.push(v1);
      stack.push(i_val);
      stack.push(j_val);
      return state.eval_error("OUT OF BOUNDS", w)
    }
  }
  if i >= j {
    let vw = state.pool.get_vword(0);
    state.current().stack.push(v1);
    state.push_quoted(Value::Word(vw));
    state.pool.add_val(i_val);
    state.pool.add_val(j_val);
    return state
  }
  let int2 = match iter.next() {
    Some((i, _)) => i,
    None => string.len(),
  };
  let mut vw = state.pool.get_vword(int2 - int1);
  vw.str_word.push_str(&string[int1..int2]);
  state.pool.add_val(i_val);
  state.pool.add_val(j_val);
  state.current().stack.push(v1);
  state.push_quoted(Value::Word(vw));
  state
}

pub fn cog_reverse(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  for c in vw.str_word.chars().rev() {
    new_vword.str_word.push(c);
  }
  state.pool.add_val(v.value_stack().pop().unwrap());
  v.value_stack().push(Value::Word(new_vword));
  state.current().stack.push(v);
  state
}

pub fn cog_word_questionmark(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let Some(v) = state.current().stack.pop() else { return state.eval_error("TOO FEW ARGUMENTS", w) };
  let mut vword = state.pool.get_vword(1);
  if v.value_stack_ref().len() == 1 {
    if v.value_stack_ref().first().unwrap().is_word() {
      vword.str_word.push('t') }}
  state.current().stack.push(v);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_ctoi(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
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
  let mut chars = vword_char.str_word.chars();
  let c = chars.next();
  if c.is_none() || chars.next().is_some() {
    stack.push(v);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  };
  let Some(math) = state.get_math() else {
    state.current().stack.push(v);
    return state.eval_error("MATH BASE ZERO", w)
  };
  match math.math().itos(c.unwrap() as isize, &mut state) {
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

pub fn cog_itoc(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let i = get_unsigned!(state, w, u32, ACTIVE, "INVALID CHAR VALUE");
  let Some(c) = char::from_u32(i) else {
    return state.eval_error("INVALID CHAR VALUE", w)
  };
  let vword = state.current().stack.last_mut().unwrap().value_stack().first_mut().unwrap().vword_mut();
  vword.str_word.clear();
  vword.str_word.push(c);
  state
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "concat", cog_concat);
  add_builtin!(state, "unconcat", cog_unconcat);
  add_builtin!(state, "cut", cog_cut);
  add_builtin!(state, "ccut", cog_ccut);
  add_builtin!(state, "substr", cog_substr);
  add_builtin!(state, "csubstr", cog_csubstr);
  add_builtin!(state, "len", cog_len);
  add_builtin!(state, "clen", cog_clen);
  add_builtin!(state, "cat", cog_cat);
  add_builtin!(state, "nth", cog_nth);
  add_builtin!(state, "replace", cog_replace);
  add_builtin!(state, "creplace", cog_creplace);
  add_builtin!(state, "slice", cog_slice);
  add_builtin!(state, "cslice", cog_cslice);
  add_builtin!(state, "reverse", cog_reverse);
  add_builtin!(state, "word?", cog_word_questionmark);
  add_builtin!(state, "ctoi", cog_ctoi);
  add_builtin!(state, "itoc", cog_itoc);
}
