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

fn serialize_value(state: &mut CognitionState, v2: Value, v3: Value, writer: &mut dyn Write) -> Option<(Value, Value, &'static str)> {
  let format_name = &v3.value_stack_ref().first().unwrap().vword_ref().str_word;
  let format = Some(format_name);
  let func = get_from_data_formats!(
    format_name, format, 5, f,
    { return Some((v2, v3, "INVALID SERDE FORMAT")) },
    { unreachable!() }, _ext,
    { unreachable!() }
  );
  match func(&v2, writer) {
    Ok(_) => {
      state.pool.add_val(v2);
      state.pool.add_val(v3);
      None
    },
    Err(_) => Some((v2, v3, "SERIALIZATION ERROR"))
  }
}

pub fn cog_serialize(mut state: CognitionState, w: Option<&Value>) -> CognitionState {
  if state.current().stack.len() < 3 { return state.eval_error("TOO FEW ARGUMENTS", w) }
  let v3 = get_word!(state, w);
  let stack = &mut state.current().stack;
  let v2 = stack.pop().unwrap();
  let mut v1 = stack.pop().unwrap();

  if v1.value_stack_ref().len() != 1 {
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

pub fn add_builtins(state: &mut CognitionState) {
  add_builtin!(state, "serialize", cog_serialize);
  // add_builtin!(state, "deserialize", cog_drop);
  // add_builtin!(state, "serde-save", cog_swap);
  // add_builtin!(state, "serde-load", cog_dup);
  // add_builtin!(state, "describe-fllibs", cog_ssize);
  // add_builtin!(state, "serialize-map", cog_serialize_map);
  // add_builtin!(state, "serde-load-fllibs", cog_serde_load_fllibs);
}
