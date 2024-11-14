pub const BLK: &[u8] = b"\x1B[30m";
pub const RED: &[u8] = b"\x1B[31m";
pub const GRN: &[u8] = b"\x1B[32m";
pub const YEL: &[u8] = b"\x1B[33m";
pub const BLU: &[u8] = b"\x1B[34m";
pub const MAG: &[u8] = b"\x1B[35m";
pub const CYN: &[u8] = b"\x1B[36m";
pub const WHT: &[u8] = b"\x1B[37m";

pub const HBLK: &[u8] = b"\x1B[90m";

pub const COLOR_RESET: &[u8] = b"\x1B[0m";

pub const DEFAULT_STACK_SIZE: usize = 24;
pub const DEFAULT_STRING_LENGTH: usize = 24;
pub const DEFAULT_BUFFER_CAPACITY: usize = 576;
pub const DEFAULT_WORD_TABLE_SIZE: usize = 576;
pub const DEFAULT_FALIASES_SIZE: usize = 24;
pub const DEFAULT_BASE: usize = 24;
pub const BUILTINS_SIZE: usize = 192;

pub const EVAL: crate::Value = crate::Value::Control(crate::VControl::Eval);
pub const RETURN: crate::Value = crate::Value::Control(crate::VControl::Return);
pub const GHOST: crate::Value = crate::Value::Control(crate::VControl::Ghost);

pub const DATA_FORMATS: [&str; 18] = [
  "JSON", "Postcard", "CBOR", "YAML", "MessagePack", "TOML", "Pickle", "RON", "BSON", "Avro", "JSON5", "URL", "S-expression", "D-Bus", "FlexBuffers", "Bencode", "DynamoDB", "CSV"
];

pub const BUILTIN_CUSTOM_DESERIALIZERS: [(&str, crate::DeserializeFn<dyn crate::Custom>); 7] = [
  ("cognition::Void",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
  ("cognition::builtins::io::ReadWriteCustom",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
  ("cognition::builtins::io::FileCustom",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
  ("cognition::builtins::io::ReadCustom",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
  ("cognition::builtins::io::WriteCustom",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
  ("cognition::builtins::io::BufReadCustom",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
  ("cognition::builtins::io::BufWriteCustom",
   (|deserializer| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
   )) as crate::DeserializeFn<dyn crate::Custom>),
];

#[macro_export]
macro_rules! bad_value_err {
  () => { panic!("Bad value on stack") };
}

#[macro_export]
macro_rules! default_fprint_error {
  ($e:expr) => {
    let _ = std::io::stderr().write("Value::fprint(): error: ".as_bytes());
    let _ = std::io::stderr().write($e.as_bytes());
  }
}
#[macro_export]
macro_rules! fwrite_check_byte {
  ($f:expr,$s:expr,$n:ident) => {{
    let n = $n;
    match $f.write($s) {
      Ok(1) => { false },
      Ok(_) => {
        $crate::default_fprint_error!("not all bytes could be written");
        $crate::default_fprint_error!(format!("wrote {n} bytes"));
        true
      },
      Err(e) => {
        $crate::default_fprint_error!(format!("{e}"));
        true
      },
    }
  }}
}

#[macro_export]
macro_rules! fwrite_check {
  ($f:expr,$s:expr) => {
    let s: &[u8] = $s;
    for n in 0..s.len() {
      if $crate::fwrite_check_byte!($f, &s[n..n+1], n) { break }
    }
  }
}

#[macro_export]
macro_rules! fwrite_check_pretty {
  ($f:expr,$s:expr) => {{
    let s: &[u8] = $s;
    for n in 0..s.len() {
      match s[n] {
        b'\n' => {
          if $crate::fwrite_check_byte!($f, b"\\", n) { break }
          if $crate::fwrite_check_byte!($f, b"n", n)  { break }
        },
        b'\t' => {
          if $crate::fwrite_check_byte!($f, b"\\", n) { break }
          if $crate::fwrite_check_byte!($f, b"t", n)  { break }
        },
        b'\r' => {
          if $crate::fwrite_check_byte!($f, b"\\", n) { break }
          if $crate::fwrite_check_byte!($f, b"r", n)  { break }
        },
        b'\'' => {
          if $crate::fwrite_check_byte!($f, b"\\", n) { break }
          if $crate::fwrite_check_byte!($f, b"'", n)  { break }
        },
        _ => {
          if $crate::fwrite_check_byte!($f, &s[n..n+1], n) { break }
        },
      }
    };
  }}
}


#[macro_export]
macro_rules! ensure_foreign_library {
  ($state:ident,$lib:ident) => {
    let mut fllibs = $state.fllibs.take();
    if fllibs.is_none() {
      fllibs = Some(ForeignLibraries::new());
    }
    if let Some(fd) = fllibs.as_mut().unwrap().get_mut(&$lib.lib_name) {
      $state.eval_error_mut("FLLIB EXISTS", None);
      $state.fllibs = fllibs;
      return
    } else {
      let key = $state.string_copy(&$lib.lib_name);
      let fd = ForeignLibrary {
        registry: std::collections::BTreeMap::new(),
        functions: $state.pool.get_functions($crate::macros::DEFAULT_STACK_SIZE),
        library: $lib.clone()
      };
      fllibs.as_mut().unwrap().insert(key, fd);
    }
    $state.fllibs = fllibs;
  }
}

#[macro_export]
macro_rules! register_custom {
  ($state:ident,$lib:ident,$custom:ty) => {
    let typename = <$custom as $crate::CustomTypeData>::custom_type_name();
    let mut new_typename = $state.pool.get_string(typename.len());
    new_typename.push_str(typename);
    let fllib_data = $state.fllibs.as_mut().unwrap().get_mut(&$lib.lib_name).unwrap();
    fllib_data.registry.insert(new_typename, <$custom as $crate::CustomTypeData>::deserialize_fn())
  }
}

/// build_macro! ensures that the macro stack requested from the pool is of appropriate
/// length no matter the number of function pointer arguments. It does this by keeping
/// a running 'Peano' count (0+1+1+1+..) as it recursively reverses the order of those
/// arguments. It then passes the reversed list to itself to recursively build the macro.
#[macro_export]
macro_rules! build_macro {
  // base case
  ($state:ident,$n:expr) => {
    $state.pool.get_vmacro($n)
  };
  (WORD,$state:ident,$lib:ident,$n:expr) => {{
    build_macro!($state, $n)
  }};
  // handle recursion
  ($state:ident,$n:expr,EVAL $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!($state, $n $(,$fi)*);
    vmacro.macro_stack.push(EVAL);
    vmacro
  }};
  ($state:ident,$n:expr,RETURN $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!($state, $n $(,$fi)*);
    vmacro.macro_stack.push(RETURN);
    vmacro
  }};
  ($state:ident,$n:expr,GHOST $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!($state, $n $(,$fi)*);
    vmacro.macro_stack.push(GHOST);
    vmacro
  }};
  ($state:ident,$n:expr,$fn:ident $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!($state, $n $(,$fi)*);
    let mut v = $state.pool.get_vfllib($fn);
    // will panic if state.builtins grows beyond u32::MAX
    v.key = $state.builtins.len() as u32;
    $state.builtins.push($fn.clone());
    vmacro.macro_stack.push($crate::Value::FLLib(v));
    vmacro
  }};
  (WORD,$state:ident,$lib:ident,$n:expr,EVAL $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!(WORD,$state,$lib, $n $(,$fi)*);
    vmacro.macro_stack.push(EVAL);
    vmacro
  }};
  (WORD,$state:ident,$lib:ident,$n:expr,RETURN $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!(WORD,$state,$lib, $n $(,$fi)*);
    vmacro.macro_stack.push(RETURN);
    vmacro
  }};
  (WORD,$state:ident,$lib:ident,$n:expr,GHOST $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!(WORD,$state,$lib, $n $(,$fi)*);
    vmacro.macro_stack.push(GHOST);
    vmacro
  }};
  (WORD,$state:ident,$lib:ident,$n:expr,$fn:ident $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!(WORD,$state,$lib, $n $(,$fi)*);

    let mut v = $state.pool.get_vfllib($fn);
    v.library = Some($lib.clone());
    let fllib_data = $state.fllibs.as_mut().unwrap().get_mut(&$lib.lib_name).unwrap();
    // will panic if fllib_data.functions grows beyond u32::MAX
    v.key = fllib_data.functions.len() as u32;
    fllib_data.functions.push($fn.clone());

    vmacro.macro_stack.push($crate::Value::FLLib(v));
    vmacro
  }};
  // reverse and count arguments
  ($state:ident,$n:expr,[] $($fr:ident)*) => {
    build_macro!($state, $n $(,$fr)*)
  };
  ($state:ident,$n:expr,[$fn:ident $($fi:ident)*] $($fr:ident)*) => {
    build_macro!($state, $n + 1, [$($fi)*] $fn $($fr)*)
  };
  (WORD,$state:ident,$lib:ident,$lib_name:ident,$n:expr,[] $($fr:ident)*) => {
    build_macro!(WORD,$state,$lib, $n $(,$fr)*)
  };
  (WORD,$state:ident,$lib:ident,$n:expr,[$fn:ident $($fi:ident)*] $($fr:ident)*) => {
    build_macro!(WORD,$state,$lib, $n + 1, [$($fi)*] $fn $($fr)*)
  }
}
/// add_word!(state: CognitionState, name: &'static str, f1, f2, ..., fn: CognitionFunction)
/// mutates state and inserts a macro word containing f1, f2, ..., fn as vfllibs into state
/// current stack's word_table. If only one CognitionFunction parameter was given, then the
/// resulting vfllib str_word value is derived from 'name'. Otherwise, they are all nameless.
#[macro_export]
macro_rules! add_builtin {
  ($state:ident,$name:literal,EVAL) => {
    let mut vmacro = build_macro!($state, 1);
    vmacro.macro_stack.push(EVAL);
    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  };
  ($state:ident,$name:literal,RETURN) => {
    let mut vmacro = build_macro!($state, 1);
    vmacro.macro_stack.push(RETURN);
    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  };
  ($state:ident,$name:literal,GHOST) => {
    let mut vmacro = build_macro!($state, 1);
    vmacro.macro_stack.push(GHOST);
    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  };
  ($state:ident,$name:literal,$f:ident) => {
    let mut vmacro = build_macro!($state, 1);

    let mut vfllib = $state.pool.get_vfllib($f);
    vfllib.str_word = Some(String::from($name));
    // will panic if state.builtins grows beyond u32::MAX
    vfllib.key = $state.builtins.len() as u32;
    $state.builtins.push($f.clone());
    vmacro.macro_stack.push($crate::Value::FLLib(vfllib));

    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  };
  ($state:ident,$name:literal$ (,$f:ident)*) => {
    let vmacro = build_macro!($state, 0, [$($f)*]);
    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  }
}
#[macro_export]
macro_rules! add_word {
  ($state:ident,$lib:ident,$name:literal,EVAL) => {
    add_builtin!($state, $name, EVAL);
  };
  ($state:ident,$lib:ident,$name:literal,RETURN) => {
    add_builtin!($state, $name, RETURN);
  };
  ($state:ident,$lib:ident,$name:literal,GHOST) => {
    add_builtin!($state, $name, GHOST);
  };
  ($state:ident,$lib:ident,$name:literal,$f:ident) => {
    let mut vmacro = build_macro!(WORD, $state, $lib, 1);

    let mut vfllib = $state.pool.get_vfllib($f);
    vfllib.str_word = Some(String::from($name));
    vfllib.library = Some($lib.clone());
    let fllib_data = $state.fllibs.as_mut().unwrap().get_mut(&$lib.lib_name).unwrap();
    // will panic if fllib_data.functions grows beyond u32::MAX
    vfllib.key = fllib_data.functions.len() as u32;
    fllib_data.functions.push($f.clone());

    vmacro.macro_stack.push($crate::Value::FLLib(vfllib));

    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  };
  ($state:ident,$lib:ident,$name:literal$ (,$f:ident)*) => {
    let vmacro = build_macro!(WORD,$state,$lib, 0, [$($f)*]);
    $state.def($crate::Value::Macro(vmacro), std::string::String::from($name));
  }
}

#[macro_export]
macro_rules! ensure_quoted {
  ($state:ident,$stack:expr) => {
    for val in $stack.iter_mut() {
      if !(val.is_stack() || val.is_macro()) {
        let new_val = $state.pool.get_vstack(1);
        let old_val = std::mem::replace(val, Value::Stack(new_val));
        val.vstack_mut().container.stack.push(old_val);
      }
    }
  }
}

#[macro_export]
macro_rules! get_char {
  ($state:ident,$c:pat,$w:ident) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let Some($c) = iter.next() else { return $state.eval_error("BAD ARGUMENT TYPE", $w) };
    if iter.next().is_some() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
}

#[macro_export]
macro_rules! get_char_option {
  ($state:ident,$c:pat,$w:ident) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let tmp = iter.next();
    let $c = tmp;
    if tmp.is_some() && iter.next().is_some() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
}

#[macro_export]
macro_rules! get_word {
  ($state:ident,$w:ident) => {{
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    cur.stack.pop().unwrap()
  }};
}

#[macro_export]
macro_rules! get_2_words {
  ($state:ident,$w:ident) => {{
    let cur = $state.current();
    if cur.stack.len() < 2 { return $state.eval_error("TOO FEW ARGUMENTS", $w) }
    let v2 = cur.stack.pop().unwrap();
    let v1 = cur.stack.last().unwrap();
    if v1.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
      cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word2_v = &v2.value_stack_ref()[0];
    let word1_v = &v1.value_stack_ref()[0];
    if !word2_v.is_word() || !word1_v.is_word() {
      cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    (cur.stack.pop().unwrap(), v2)
  }};
}

// note: do not use usize (use isize instead)
#[macro_export]
macro_rules! get_unsigned {
  ($state:ident,$w:ident,$type:ty,ACTIVE,$err:literal) => {{
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let stack = v.value_stack_ref();
    if stack.len() != 1 {
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val = &stack[0];
    let $crate::Value::Word(vword) = word_val else {
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(ref math) = cur.math else {
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.base() == 0 {
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i = match math.stoi(&vword.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < 0 {
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => return $state.eval_error(e, $w),
    };
    i
  }};
  ($state:ident,$w:ident,$type:ty,ACTIVE) => {
    get_int!($state, $w, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$type:ty) => {{
    let i = get_int!($state, $w, $type, ACTIVE);
    let v = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v);
    i
  }};
  ($state:ident,$w:ident) => {
    get_int!($state, $w, i32)
  };
}

#[macro_export]
macro_rules! get_int {
  ($state:ident,$w:ident,$type:ty,ACTIVE,$err:literal) => {{
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let stack = v.value_stack_ref();
    if stack.len() != 1 {
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val = &stack[0];
    let $crate::Value::Word(vword) = word_val else {
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(ref math) = cur.math else {
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.base() == 0 {
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i = match math.stoi(&vword.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < <$type>::MIN as isize {
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => return $state.eval_error(e, $w),
    };
    i
  }};
  ($state:ident,$w:ident,$type:ty,ACTIVE) => {
    get_int!($state, $w, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$type:ty) => {{
    let i = get_int!($state, $w, $type, ACTIVE);
    let v = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v);
    i
  }};
  ($state:ident,$w:ident) => {
    get_int!($state, $w, i32)
  };
}

#[macro_export]
macro_rules! get_2_ints {
  ($state:ident,$w:ident,$type:ty,ACTIVE,$err:literal) => {{
    let cur = $state.current();
    let Some(v2) = cur.stack.pop() else { return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let Some(v1) = cur.stack.pop() else {
      cur.stack.push(v2);
      return $state.eval_error("TOO FEW ARGUMENTS", $w);
    };
    let stack1 = v1.value_stack_ref();
    if stack1.len() != 1 {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val1 = &stack1[0];
    let $crate::Value::Word(vword1) = word_val1 else {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let stack2 = v2.value_stack_ref();
    if stack2.len() != 1 {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val2 = &stack2[0];
    let $crate::Value::Word(vword2) = word_val2 else {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(ref math) = cur.math else {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.base() == 0 {
      cur.stack.push(v1); cur.stack.push(v2);
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i1 = match math.stoi(&vword1.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < 0 {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error(e, $w)
      },
    };
    let i2 = match math.stoi(&vword2.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < 0 {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        cur.stack.push(v1); cur.stack.push(v2);
        return $state.eval_error(e, $w)
      },
    };
    $state.current().stack.push(v1);
    $state.current().stack.push(v2);
    (i1, i2)
  }};
  ($state:ident,$w:ident,$type:ty,ACTIVE) => {
    get_2_ints!($state, $w, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$type:ty) => {{
    let (i1, i2) = get_2_ints!($state, $w, $type, ACTIVE);
    let v2 = $state.current().stack.pop().unwrap();
    let v1 = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v1);
    $state.pool.add_val(v2);
    (i1, i2)
  }};
  ($state:ident,$w:ident) => {
    get_2_ints!($state, $w, i32)
  };
}
