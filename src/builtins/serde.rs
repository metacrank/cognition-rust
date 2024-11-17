use crate::*;
use super::io::*;
use std::io::Write;
use std::fs::File;

macro_rules! serialize_value {
  ($state:ident,$w:ident,$v1:ident,$v2:ident,$v3:ident,$writer:expr) => {
    if let Some((v2, v3, erst)) = serialize_value(&mut $state, $v2, $v3, $writer) {
      $state.current().stack.push($v1);
      $state.current().stack.push(v2);
      $state.current().stack.push(v3);
      return $state.eval_error(erst, $w)
    }
  }
}

pub fn serialize_value(state: &mut CognitionState, vdata: Value, vformat: Value, writer: &mut dyn Write) -> Option<(Value, Value, &'static str)> {
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 6, f,
    { return Some((vdata, vformat, "INVALID SERDE FORMAT")) },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  match func(&vdata.value_stack_ref().first().unwrap(), writer) {
    Ok(_) => {
      state.pool.add_val(vdata);
      state.pool.add_val(vformat);
      None
    },
    Err(_) => Some((vdata, vformat, "SERIALIZATION FAILED"))
  }
}

pub fn cog_serialize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_word!(state, w);
  let v1 = state.current().stack.pop().unwrap();
  if v1.value_stack_ref().len() != 1 {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let format_name = &v2.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 6, f,
    {
      state.current().stack.push(v1);
      state.current().stack.push(v2);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let mut vec = Vec::<u8>::with_capacity(DEFAULT_STRING_LENGTH);
  if func(&v1.value_stack_ref().first().unwrap(), &mut vec).is_err() {
    state.current().stack.push(v1);
    state.current().stack.push(v2);
    return state.eval_error("SERIALIZATION FAILED", w)
  }
  let string = match String::from_utf8(vec) {
    Ok(s) => s,
    Err(_) => {
      state.current().stack.push(v1);
      state.current().stack.push(v2);
      return state.eval_error("INVALID STRING", w)
    }
  };
  let mut vword = state.pool.get_vword(0);
  let string = std::mem::replace(&mut vword.str_word, string);
  state.pool.add_string(string);
  state.pool.add_val(v1);
  state.pool.add_val(v2);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_fserialize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v3 = get_word!(state, w);
  let stack = &mut state.current().stack;
  let v2 = stack.pop().unwrap();
  let mut v1 = stack.pop().unwrap();

  if v1.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
    stack.push(v1);
    stack.push(v2);
    stack.push(v3);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let vwriter = v1.value_stack().first_mut().unwrap();
  match vwriter {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        serialize_value!(state, w, v1, v2, v3, &mut file.file.as_mut().unwrap());
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        serialize_value!(state, w, v1, v2, v3, &mut writer.writer.as_mut().unwrap());
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        serialize_value!(state, w, v1, v2, v3, &mut stream.stream.as_mut().unwrap());
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        serialize_value!(state, w, v1, v2, v3, &mut bufwriter.bufwriter.as_mut().unwrap());
      } else {
        stack.push(v1);
        stack.push(v2);
        stack.push(v3);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v1);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::create_new(&vword.str_word) {
        serialize_value!(state, w, v1, v2, v3, &mut file);
      } else if let Ok(mut file) = File::open(&vword.str_word) {
        serialize_value!(state, w, v1, v2, v3, &mut file);
      } else {
        stack.push(v1);
        stack.push(v2);
        stack.push(v3);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v1);
    },
    _ => return {
      stack.push(v1);
      stack.push(v2);
      stack.push(v3);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_deserialize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let (vdata, vformat) = get_2_words!(state, w);
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let deserialize_fn = get_from_data_formats!(
    format_name, format, 7, f,
    {
      state.current().stack.push(vdata);
      state.current().stack.push(vformat);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let data = &vdata.value_stack_ref().first().unwrap().vword_ref().str_word;
  match deserialize_fn(data, &mut state) {
    Ok(val) => {
      state.push_quoted(val);
      state.pool.add_val(vdata);
      state.pool.add_val(vformat);
      state
    },
    Err(_) => {
      state.current().stack.push(vdata);
      state.current().stack.push(vformat);
      state.eval_error("DESERIALIZATION FAILED", w)
    }
  }
}

pub fn cog_state(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vformat = get_word!(state, w);
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 4, f,
    {
      state.current().stack.push(vformat);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let mut vec = Vec::<u8>::with_capacity(DEFAULT_STRING_LENGTH);
  if func(&state, &mut vec).is_err() {
    state.current().stack.push(vformat);
    return state.eval_error("SERIALIZATION FAILED", w)
  }
  let string = match String::from_utf8(vec) {
    Ok(s) => s,
    Err(_) => {
      state.current().stack.push(vformat);
      return state.eval_error("INVALID STRING", w)
    }
  };
  let mut vword = state.pool.get_vword(0);
  let string = std::mem::replace(&mut vword.str_word, string);
  state.pool.add_string(string);
  state.pool.add_val(vformat);
  state.push_quoted(Value::Word(vword));
  state
}

macro_rules! serialize_state {
  ($state:ident,$w:ident,$v1:ident,$v2:ident,$writer:expr) => {
    if let Some((v2, erst)) = serialize_state(&mut $state, $v2, $writer) {
      $state.current().stack.push($v1);
      $state.current().stack.push(v2);
      return $state.eval_error(erst, $w)
    }
  }
}

pub fn serialize_state(state: &mut CognitionState, vformat: Value, writer: &mut dyn Write) -> Option<(Value, &'static str)> {
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 4, f,
    { return Some((vformat, "INVALID SERDE FORMAT")) },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  match func(state, writer) {
    Ok(_) => { state.pool.add_val(vformat); None },
    Err(e) => { println!("{e}"); Some((vformat, "SERIALIZATION FAILED")) }
  }
}

pub fn cog_fstate(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v2 = get_word!(state, w);
  let stack = &mut state.current().stack;
  let mut v1 = stack.pop().unwrap();
  if v1.value_stack_ref().len() != 1 {
    stack.push(v1);
    stack.push(v2);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let vwriter = v1.value_stack().first_mut().unwrap();
  match vwriter {
    Value::Custom(vcustom) => {
      let custom = &mut vcustom.custom;
      if let Some(file) = custom.as_any_mut().downcast_mut::<FileCustom>() {
        serialize_state!(state, w, v1, v2, &mut file.file.as_mut().unwrap());
      } else if let Some(writer) = custom.as_any_mut().downcast_mut::<WriteCustom>() {
        serialize_state!(state, w, v1, v2, &mut writer.writer.as_mut().unwrap());
      } else if let Some(stream) = custom.as_any_mut().downcast_mut::<ReadWriteCustom>() {
        serialize_state!(state, w, v1, v2, &mut stream.stream.as_mut().unwrap());
      } else if let Some(bufwriter) = custom.as_any_mut().downcast_mut::<BufWriteCustom>() {
        serialize_state!(state, w, v1, v2, &mut bufwriter.bufwriter.as_mut().unwrap());
      } else {
        stack.push(v1);
        stack.push(v2);
        return state.eval_error("BAD ARGUMENT TYPE", w)
      }
      state.current().stack.push(v1);
    },
    Value::Word(vword) => {
      if let Ok(mut file) = File::create_new(&vword.str_word) {
        serialize_state!(state, w, v1, v2, &mut file);
      } else if let Ok(mut file) = File::options().write(true).create(true).open(&vword.str_word) {
        serialize_state!(state, w, v1, v2, &mut file);
      } else {
        stack.push(v1);
        stack.push(v2);
        return state.eval_error("INVALID FILENAME", w)
      };
      state.pool.add_val(v1);
    },
    _ => return {
      stack.push(v1);
      stack.push(v2);
      state.eval_error("BAD ARGUMENT TYPE", w)
    },
  }
  state
}

pub fn cog_restate(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let (vdata, vformat) = get_2_words!(state, w);

  let data = &vdata.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);

  let deserialize_fn = get_from_data_formats!(
    format_name, format, 2, f,
    {
      state.current().stack.push(vdata);
      state.current().stack.push(vformat);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let new_state = serde::cogstate_init();
  if let Ok(s) = deserialize_fn(data, true, new_state) { return s }
  state.current().stack.push(vdata);
  state.current().stack.push(vformat);
  state.eval_error("DESERIALIZATION FAILED", w)
}

pub fn cog_describe_fllibs(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  let vformat = get_word!(state, w);
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 5, f,
    {
      state.current().stack.push(vformat);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let mut vec = Vec::<u8>::with_capacity(DEFAULT_STRING_LENGTH);
  if func(&state.fllibs, &mut vec).is_err() {
    state.current().stack.push(vformat);
    return state.eval_error("SERIALIZATION FAILED", w)
  }
  let string = match String::from_utf8(vec) {
    Ok(s) => s,
    Err(_) => {
      state.current().stack.push(vformat);
      return state.eval_error("INVALID STRING", w)
    }
  };
  let mut vword = state.pool.get_vword(0);
  let string = std::mem::replace(&mut vword.str_word, string);
  state.pool.add_string(string);
  state.pool.add_val(vformat);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_serialize_map(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let vformat = get_word!(state, w);
  let stack = &mut state.current().stack;
  let mut vmap = stack.pop().unwrap();
  let requirement = |v: &Value| {
    if v.is_stack() || v.is_macro() {
      if v.value_stack_ref().len() > 0 {
        v.value_stack_ref().iter().all(|w| w.is_word())
      } else { false }
    } else { false }
  };
  if !vmap.value_stack().iter().all(requirement) {
    stack.push(vmap);
    stack.push(vformat);
    return state.eval_error("BAD ARGUMENT TYPE", w)
  }
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 8, f,
    {
      state.current().stack.push(vformat);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let mut vec = Vec::<u8>::with_capacity(DEFAULT_STRING_LENGTH);
  if func(&vmap, &mut vec).is_err() {
    state.current().stack.push(vformat);
    return state.eval_error("SERIALIZATION FAILED", w)
  }
  let string = match String::from_utf8(vec) {
    Ok(s) => s,
    Err(_) => {
      state.current().stack.push(vformat);
      return state.eval_error("INVALID STRING", w)
    }
  };
  let mut vword = state.pool.get_vword(0);
  let string = std::mem::replace(&mut vword.str_word, string);
  state.pool.add_string(string);
  state.pool.add_val(vformat);
  state.pool.add_val(vmap);
  state.push_quoted(Value::Word(vword));
  state
}

pub fn cog_load_fllibs(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 2 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let (vdata, vformat) = get_2_words!(state, w);
  let format_name = &vformat.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let deserialize_fn = get_from_data_formats!(
    format_name, format, 3, f,
    {
      state.current().stack.push(vdata);
      state.current().stack.push(vformat);
      return state.eval_error("INVALID SERDE FORMAT", w)
    },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  let data = &vdata.value_stack_ref().first().unwrap().vword_ref().str_word;
  match deserialize_fn(data, state) {
    Ok(state) => state,
    Err(mut e) => {
      e.0.current().stack.push(vdata);
      e.0.current().stack.push(vformat);
      match format!("{}", e.1).as_str() {
        "INVALID FILENAME" => e.0.eval_error("INVALID FILENAME", w),
        "INVALID FLLIB" => e.0.eval_error("INVALID FLLIB", w),
        _ => e.0.eval_error("DESERIALIZATION FAILED", w)
      }
    }
  }
}

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "serialize", cog_serialize);
  add_builtin!(state, "fserialize", cog_fserialize);
  add_builtin!(state, "deserialize", cog_deserialize);
  add_builtin!(state, "state", cog_state);
  add_builtin!(state, "fstate", cog_fstate);
  add_builtin!(state, "restate", cog_restate);
  // describe-fllibs could be replaced with ( get-fllibs serialize-map )
  add_builtin!(state, "describe-fllibs", cog_describe_fllibs);
  add_builtin!(state, "serialize-map", cog_serialize_map);
  add_builtin!(state, "load-fllibs", cog_load_fllibs);
}
