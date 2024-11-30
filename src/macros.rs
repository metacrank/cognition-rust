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
pub const DEFAULT_OP_SIZE: usize = 576;
pub const DEFAULT_OPS_TABLE_SIZE: usize = 24;
pub const DEFAULT_BASE: usize = 24;
pub const BUILTINS_SIZE: usize = 192;

pub const EVAL: crate::Value = crate::Value::Control(crate::VControl::Eval);
pub const RETURN: crate::Value = crate::Value::Control(crate::VControl::Return);
pub const GHOST: crate::Value = crate::Value::Control(crate::VControl::Ghost);

#[macro_export]
macro_rules! get_from_data_formats {
  ($filename:ident,$format:ident,$index:tt,$f:tt,$fail1:block,$fail2:block,$ext_slice:tt,$fail3:block) => {
    match $format {
      Some($f) => {
        let mut iter = DATA_FORMATS.iter();
        loop {
          let Some(element) = iter.next() else {
            $fail1
          };
          if *element.0 == *$f {
            break element.$index.clone()
          }
        }
      },
      None => {
        let mut rev = $filename.char_indices().rev();
        let dot_idx = loop {
          let Some((i, c)) = rev.next() else {
            $fail2
          };
          if c == '.' {
            break i;
          }
        };
        let $ext_slice = &$filename[dot_idx..];
        let mut iter = DATA_FORMATS.iter();
        loop {
          let Some(element) = iter.next() else {
            $fail3
          };
          if *element.1 == *$ext_slice {
            break element.$index.clone()
          }
        }
      }
    }
  };
  ($filename:ident,$format:ident,$index:tt,MAIN) => {{
    get_from_data_formats!($filename, $format, $index, f, {
      println!("{}: invalid format -- '{}'", binary_name(), f);
      println!("Run '{} --list-formats' for a list of supported formats and file extensions", binary_name());
      return Err(try_help(2))
    }, {
      println!("{}: could not infer format from nonexistent file extension", binary_name());
      println!("Please specify a format with '--format|--save-format FORMAT' or include a file extension.");
      return Err(try_help(2))
    }, ext_slice, {
      println!("{}: invalid file extension -- \"{}\"", binary_name(), ext_slice);
      println!("Run '{} --list-formats' for a list of supported formats and file extensions", binary_name());
      return Err(try_help(2))
    })
  }}
}

macro_rules! get_serde_fn {
  ($state:ident,$filename:ident,$format:ident,$index:tt,$f:tt,$fail1:block,$fail2:block,$ext_slice:tt,$fail3:block) => {
    'func: {
      if let Some($f) = $format {
        for element in $state.serde.serdes.iter() {
          if *element.0 == *$f {
            break 'func element.$index.clone()
          }
        }
      }
      get_from_data_formats!{ $filename,$format,$index,$f,$fail1,$fail2,$ext_slice,$fail3 }
    }
  }
}
macro_rules! get_serialize_fn {
  ($state:ident,$filename:ident,$format:ident,$index:tt,$f:tt,$fail1:block,$fail2:block,$ext_slice:tt,$fail3:block) => {
    'func: {
      if let Some($f) = $format {
        for element in $state.serde.serdes.iter() {
          if *element.0 == *$f {
            break 'func element.$index.clone()
          }
        }
        for element in $state.serde.serializers.iter() {
          if *element.0 == *$f {
            break 'func element.$index.clone()
          }
        }
      }
      get_from_data_formats!{ $filename,$format,$index,$f,$fail1,$fail2,$ext_slice,$fail3 }
    }
  }
}
macro_rules! get_deserialize_fn {
  ($state:ident,$filename:ident,$format:ident,$index:tt,$f:tt,$fail1:block,$fail2:block,$ext_slice:tt,$fail3:block) => {
    'func: {
      if let Some($f) = $format {
        for element in $state.serde.serdes.iter() {
          if *element.0 == *$f {
            break 'func element.$index.clone()
          }
        }
        for element in $state.serde.deserializers.iter() {
          if *element.0 == *$f {
            break 'func element.$index.clone()
          }
        }
      }
      get_from_data_formats!{ $filename,$format,$index,$f,$fail1,$fail2,$ext_slice,$fail3 }
    }
  }
}


macro_rules! data_formats_entry {
  ($deserializer_from_str:expr,2) => {{
    (|f, i, mut state| {
      let mut deserializer = $deserializer_from_str(f);
      match crate::serde::deserialize_cognition_state_from_state(&mut deserializer, &mut state, i) {
        Ok(_) => Ok(state),
        Err(e) => Err((state, Box::new(e)))
      }
    })
  }};
  ($deserializer_from_str:expr,3) => {{
    (|fllibs, mut state| {
      let mut deserializer = $deserializer_from_str(fllibs);
      match crate::serde::serde_load_fllibs(&mut deserializer, &mut state) {
        Err(e) => Err((state, Box::new(e))),
        Ok(opt) => match opt {
          Some(e) => Err((state, Box::new(e))),
          None => Ok(state)
        }
      }
    })
  }};
  ($serializer_new:expr,4) => {{
    (|state, write| {
      let mut serializer = $serializer_new(write);
      match <crate::CognitionState as ::serde::ser::Serialize>::serialize(state, &mut serializer) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e))
      }
    })
  }};
  ($serializer_new:expr,5) => {{
    (|fllibs, write| {
      let mut serializer = $serializer_new(write);
      let libs = crate::serde::ForeignLibrariesWrapper(fllibs);
      match <crate::serde::ForeignLibrariesWrapper as ::serde::ser::Serialize>::serialize(&libs, &mut serializer) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e))
      }
    })
  }};
  ($serializer_new:expr,6) => {{
    (|val, write| {
      let mut serializer = $serializer_new(write);
      match <crate::Value as ::serde::ser::Serialize>::serialize(val, &mut serializer) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e))
      }
    })
  }};
  ($deserializer_from_str:expr,7) => {{
    (|string, state| {
      let mut deserializer = $deserializer_from_str(string);
      match <crate::Value as crate::serde::CognitionDeserialize>::cognition_deserialize(&mut deserializer, state) {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e))
      }
    })
  }};
  ($serializer_new:expr,8) => {{
    (|mapval, write| {
      let mut serializer = $serializer_new(write);
      match crate::serde::serialize_cogmap(&mut serializer, mapval) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e))
      }
    })
  }};
  ($fmt:literal,$ext:literal,$deserializer_from_str:expr,$serializer_new:expr) => {
    ($fmt, $ext,
     data_formats_entry!{ $deserializer_from_str, 2 },
     data_formats_entry!{ $deserializer_from_str, 3 },
     data_formats_entry!{ $serializer_new, 4 },
     data_formats_entry!{ $serializer_new, 5 },
     data_formats_entry!{ $serializer_new, 6 },
     data_formats_entry!{ $deserializer_from_str, 7 },
     data_formats_entry!{ $serializer_new, 8 }
    )
  }
}

macro_rules! serde_descriptor {
  ($fmt:literal,$deserializer_from_str:expr,$serializer_new:expr) => {
    ($fmt, (),
     data_formats_entry!{ $deserializer_from_str, 2 },
     data_formats_entry!{ $deserializer_from_str, 3 },
     data_formats_entry!{ $serializer_new, 4 },
     data_formats_entry!{ $serializer_new, 5 },
     data_formats_entry!{ $serializer_new, 6 },
     data_formats_entry!{ $deserializer_from_str, 7 },
     data_formats_entry!{ $serializer_new, 8 }
    )
  }
}
macro_rules! serializer_descriptor {
  ($fmt:literal,$serializer_new:expr) => {
    ($fmt, (), (), (),
     data_formats_entry!{ $serializer_new, 4 },
     data_formats_entry!{ $serializer_new, 5 },
     data_formats_entry!{ $serializer_new, 6 }, (),
     data_formats_entry!{ $serializer_new, 8 }
    )
  }
}
macro_rules! deserializer_descriptor {
  ($fmt:literal,$deserializer_from_str:expr) => {
    ($fmt, (),
     data_formats_entry!{ $deserializer_from_str, 2 },
     data_formats_entry!{ $deserializer_from_str, 3 }, (), (), (),
     data_formats_entry!{ $deserializer_from_str, 7 }, ()
    )
  }
}

pub type SerdeDescriptor = (
  &'static str, (),
  crate::CogStateDeserializeFn,
  crate::CogLibsDeserializeFn,
  crate::CogStateSerializeFn,
  crate::CogLibsSerializeFn,
  crate::CogValueSerializeFn,
  crate::CogValueDeserializeFn,
  crate::CogValueSerializeFn
);

pub type SerializerDescriptor = (
  &'static str, (), (), (),
  crate::CogStateSerializeFn,
  crate::CogLibsSerializeFn,
  crate::CogValueSerializeFn, (),
  crate::CogValueSerializeFn
);

pub type DeserializerDescriptor = (
  &'static str, (),
  crate::CogStateDeserializeFn,
  crate::CogLibsDeserializeFn, (), (), (),
  crate::CogValueDeserializeFn, ()
);

pub type DataFormatsEntry<'a> = (
  &'a str, &'a str,
  crate::CogStateDeserializeFn,
  crate::CogLibsDeserializeFn,
  crate::CogStateSerializeFn,
  crate::CogLibsSerializeFn,
  crate::CogValueSerializeFn,
  crate::CogValueDeserializeFn,
  crate::CogValueSerializeFn
);

pub const DATA_FORMATS: [DataFormatsEntry; 1] = [
  data_formats_entry!{ "JSON", ".json", serde_json::Deserializer::from_str, serde_json::Serializer::new }
  //"Postcard", "CBOR", "YAML", "MessagePack", "TOML", "Pickle", "RON", "BSON", "Avro", "JSON5", "URL", "S-expression", "D-Bus", "FlexBuffers", "Bencode", "DynamoDB", "CSV"
];

#[macro_export]
macro_rules! void_deserialize_fn {
  () => {
    (|deserializer, _state| Ok(
     Box::new(erased_serde::deserialize::<crate::Void>(deserializer)?),
    )) as crate::DeserializeFn<dyn crate::Custom>
  }
}

#[macro_export]
macro_rules! option_deserialize_fn {
  ($t:ty) => {
    (|deserializer, state| Ok(
      match <$t as crate::serde::OptionDeserialize>::option_deserialize(deserializer, state)? {
        Some(c) => Box::new(c),
        None => Box::new(crate::Void{})
      }
    ))
  }
}

pub const BUILTIN_CUSTOM_DESERIALIZERS: [(&str, crate::DeserializeFn<dyn crate::Custom>); 7] = [
  ("cognition::Void", void_deserialize_fn!{}),
  ("cognition::builtins::io::ReadWriteCustom", option_deserialize_fn!{crate::builtins::io::ReadWriteCustom}),
  ("cognition::builtins::io::FileCustom", option_deserialize_fn!{crate::builtins::io::FileCustom}),
  ("cognition::builtins::io::ReadCustom", option_deserialize_fn!{crate::builtins::io::ReadCustom}),
  ("cognition::builtins::io::WriteCustom", option_deserialize_fn!{crate::builtins::io::WriteCustom}),
  ("cognition::builtins::io::BufReadCustom", option_deserialize_fn!{crate::builtins::io::BufReadCustom}),
  ("cognition::builtins::io::BufWriteCustom", option_deserialize_fn!{crate::builtins::io::BufWriteCustom})
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

#[macro_export]
macro_rules! custom_pool_name {
  ($custom:literal) => {
    concat!(module_path!(), "::", $custom);
  }
}

#[macro_export]
macro_rules! add_custom_pool {
  ($state:ident,$name:literal,TREE) => {
    let pool = $crate::pool::CustomPool::Tree($crate::pool::VTree::new());
    add_custom_pool!($state, $name, pool);
  };
  ($state:ident,$name:literal,VEC) => {
    let pool = $crate::pool::CustomPool::Vec($crate::Stack::new());
    add_custom_pool!($state, $name, pool);
  };
  ($state:ident,$name:literal,$pool:ident) => {
    $state.pool.custom_pools.insert(format!("{}::{}", module_path!(), $name), $pool);
  }
}

#[macro_export]
macro_rules! get_from_custom_pool {
  ($pool:ident,$name:literal,$i:expr,$var:pat,$var_type:ty,$block:block,$default_block:block) => {{
    let pool_name = custom_pool_name!($name);
    get_from_custom_pool!($pool,$i,$var,$var_type, pool_name, $block,$default_block)
  }};
  ($pool:ident,$i:expr,$var:pat,$var_type:ty,$pool_name:expr,$block:block,$default_block:block) => {
    'ret: {
      if let Some(mut vcustom) = $pool.get_vcustom($pool_name, $i) {
        if let Some($var) = vcustom.custom.as_any_mut().downcast_mut::<$var_type>() {
          $block;
          break 'ret vcustom
        }
        $pool.add_vcustom(vcustom);
      }
      $default_block
    }
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
macro_rules! impl_serde_as_null {
  ($type:ty,$default:expr) => {
    impl ::serde::ser::Serialize for $type {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where S: ::serde::ser::Serializer
      { serializer.serialize_none() }
    }
    impl<'de> ::serde::de::Deserialize<'de> for $type {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where D: ::serde::de::Deserializer<'de>,
      {
        struct Visitor;
        impl<'de> ::serde::de::Visitor<'de> for Visitor {
          type Value = $type;
          fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null")
          }
          fn visit_none<E: ::serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok($default)
          }
        }
        deserializer.deserialize_option(Visitor)
      }
    }
  }
}

#[macro_export]
macro_rules! get_char {
  ($state:ident,$c:pat,$w:ident,$fail:block) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let Some($c) = iter.next() else { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) };
    if iter.next().is_some() { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
  ($state:ident,$c:pat,$w:ident) => {
    get_char!($state, $c, $w, {});
  }
}

#[macro_export]
macro_rules! get_char_option {
  ($state:ident,$c:pat,$w:ident,$fail:block) => {
    let cur = $state.current();
    let Some(v) = cur.stack.last() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let s = &word_v.vword_ref().str_word;
    let mut iter = s.chars();
    let tmp = iter.next();
    let $c = tmp;
    if tmp.is_some() && iter.next().is_some() { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let v = cur.stack.pop().unwrap();
    $state.pool.add_val(v);
  };
  ($state:ident,$c:pat,$w:ident) => {
    get_char_option!($state, $c, $w, {});
  }
}

#[macro_export]
macro_rules! get_word {
  ($state:ident,$w:ident,$fail:block,ACTIVE) => {
    let Some(v) = $state.current().stack.last() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_v = &v.value_stack_ref()[0];
    if !word_v.is_word() { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
  };
  ($state:ident,$w:ident,$fail:block) => {{
    get_word!($state,$w,$fail,ACTIVE);
    $state.current().stack.pop().unwrap()
  }};
  ($state:ident,$w:ident$(,$t:tt)?) => {
    get_word!($state, $w, {} $(,$t)? )
  }
}

#[macro_export]
macro_rules! get_2_words {
  ($state:ident,$w:ident,$fail:block) => {{
    let stack = &mut $state.current().stack;
    if stack.len() < 2 { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) }
    let v2 = stack.pop().unwrap();
    let v1 = stack.last().unwrap();
    if v1.value_stack_ref().len() != 1 || v2.value_stack_ref().len() != 1 {
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word2_v = &v2.value_stack_ref()[0];
    let word1_v = &v1.value_stack_ref()[0];
    if !word2_v.is_word() || !word1_v.is_word() {
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    (stack.pop().unwrap(), v2)
  }};
  ($state:ident,$w:ident) => {
    get_2_words!($state, $w, {})
  }
}

#[macro_export]
macro_rules! get_custom {
  ($state:ident,$w:ident,$fail:block,ACTIVE) => {
    let Some(v) = $state.current_ref().stack.last() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    if v.value_stack_ref().len() != 1 { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    if !v.value_stack_ref().first().unwrap().is_custom() { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) };
  };
  ($state:ident,$w:ident,$fail:block) => {{
    get_custom!($state,$w,$fail,ACTIVE);
    $state.current().stack.pop().unwrap()
  }};
  ($state:ident,$w:ident$(,$t:tt)?) => {
    get_custom!($state, $w, {} $(,$t)?)
  }
}

// note: do not use usize (use isize instead)
#[macro_export]
macro_rules! get_unsigned {
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE,$err:literal) => {{
    let Some(v) = $state.current().stack.pop() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let stack = v.value_stack_ref();
    if stack.len() != 1 {
      $state.current().stack.push(v);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val = &stack[0];
    let $crate::Value::Word(vword) = word_val else {
      $state.current().stack.push(v);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(math) = $state.get_math() else {
      $state.current().stack.push(v);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.math().base() == 0 {
      $state.set_math(math);
      $state.current().stack.push(v);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i = match math.math().stoi(&vword.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < 0 {
        $state.set_math(math);
        $state.current().stack.push(v);
        $fail;
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v);
        $fail;
        return $state.eval_error(e, $w)
      }
    };
    $state.set_math(math);
    $state.current().stack.push(v);
    i
  }};
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE) => {
    get_unsigned!($state, $w, $fail, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$fail:block,$type:ty) => {{
    let i = get_unsigned!($state, $w, $fail, $type, ACTIVE);
    let v = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v);
    i
  }};
  ($state:ident,$w:ident,$fail:block) => {
    get_unsigned!($state, $w, i32)
  };
  ($state:ident,$w:ident$(,$t:tt)*) => {
    get_unsigned!($state, $w, {} $(,$t)*)
  }
}

#[macro_export]
macro_rules! get_int {
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE,$err:literal) => {{
    let Some(v) = $state.current().stack.pop() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let stack = v.value_stack_ref();
    if stack.len() != 1 { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) }
    let word_val = &stack[0];
    let $crate::Value::Word(vword) = word_val else { $fail; return $state.eval_error("BAD ARGUMENT TYPE", $w) };
    let Some(math) = $state.get_math() else {
      $state.current().stack.push(v);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.math().base() == 0 {
      $state.set_math(math);
      $state.current().stack.push(v);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i = match math.math().stoi(&vword.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < <$type>::MIN as isize {
        $state.set_math(math);
        $state.current().stack.push(v);
        $fail;
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v);
        $fail;
        return $state.eval_error(e, $w)
      }
    };
    $state.set_math(math);
    $state.current().stack.push(v);
    i
  }};
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE) => {
    get_int!($state, $w, $fail, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$fail:block,$type:ty) => {{
    let i = get_int!($state, $w, $fail, $type, ACTIVE);
    let v = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v);
    i
  }};
  ($state:ident,$w:ident,$fail:block) => {
    get_int!($state, $w, $fail, i32)
  };
  ($state:ident,$w:ident$(,$t:tt)*) => {
    get_int!($state, $w, {} $(,$t)*)
  }
}

#[macro_export]
macro_rules! get_2_unsigned {
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE,$err:literal) => {{
    let stack = &mut $state.current().stack;
    let Some(v2) = stack.pop() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let Some(v1) = stack.pop() else {
      stack.push(v2);
      $fail;
      return $state.eval_error("TOO FEW ARGUMENTS", $w);
    };
    let stack1 = v1.value_stack_ref();
    if stack1.len() != 1 {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val1 = &stack1[0];
    let $crate::Value::Word(vword1) = word_val1 else {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let stack2 = v2.value_stack_ref();
    if stack2.len() != 1 {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val2 = &stack2[0];
    let $crate::Value::Word(vword2) = word_val2 else {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(math) = $state.get_math() else {
      $state.current().stack.push(v1);
      $state.current().stack.push(v2);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.math().base() == 0 {
      $state.set_math(math);
      $state.current().stack.push(v1);
      $state.current().stack.push(v2);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i1 = match math.math().stoi(&vword1.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < 0 {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error(e, $w)
      },
    };
    let i2 = match math.math().stoi(&vword2.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < 0 {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error(e, $w)
      },
    };
    $state.set_math(math);
    $state.current().stack.push(v1);
    $state.current().stack.push(v2);
    (i1, i2)
  }};
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE) => {
    get_2_unsigned!($state, $w, $fail, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$fail:block,$type:ty) => {{
    let (i1, i2) = get_2_unsigned!($state, $w, $fail, $type, ACTIVE);
    let v2 = $state.current().stack.pop().unwrap();
    let v1 = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v1);
    $state.pool.add_val(v2);
    (i1, i2)
  }};
  ($state:ident,$w:ident,$fail:block) => {
    get_2_unsigned!($state, $w, $fail, i32)
  };
  ($state:ident,$w:ident$(,$t:tt)*) => {
    get_2_unsigned!($state, $w, {} $(,$t)*)
  }
}

#[macro_export]
macro_rules! get_2_ints {
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE,$err:literal) => {{
    let stack = &mut $state.current().stack;
    let Some(v2) = stack.pop() else { $fail; return $state.eval_error("TOO FEW ARGUMENTS", $w) };
    let Some(v1) = stack.pop() else {
      stack.push(v2);
      $fail;
      return $state.eval_error("TOO FEW ARGUMENTS", $w);
    };
    let stack1 = v1.value_stack_ref();
    if stack1.len() != 1 {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val1 = &stack1[0];
    let $crate::Value::Word(vword1) = word_val1 else {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let stack2 = v2.value_stack_ref();
    if stack2.len() != 1 {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    }
    let word_val2 = &stack2[0];
    let $crate::Value::Word(vword2) = word_val2 else {
      stack.push(v1);
      stack.push(v2);
      $fail;
      return $state.eval_error("BAD ARGUMENT TYPE", $w)
    };
    let Some(math) = $state.get_math() else {
      $state.current().stack.push(v1);
      $state.current().stack.push(v2);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    };
    if math.math().base() == 0 {
      $state.set_math(math);
      $state.current().stack.push(v1);
      $state.current().stack.push(v2);
      $fail;
      return $state.eval_error("MATH BASE ZERO", $w)
    }
    let i1 = match math.math().stoi(&vword1.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < <$type>::MIN as isize {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error(e, $w)
      },
    };
    let i2 = match math.math().stoi(&vword2.str_word) {
      Ok(i) => if i > <$type>::MAX as isize || i < <$type>::MIN as isize {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error($err, $w)
      } else { i as $type },
      Err(e) => {
        $state.set_math(math);
        $state.current().stack.push(v1);
        $state.current().stack.push(v2);
        $fail;
        return $state.eval_error(e, $w)
      },
    };
    $state.set_math(math);
    $state.current().stack.push(v1);
    $state.current().stack.push(v2);
    (i1, i2)
  }};
  ($state:ident,$w:ident,$fail:block,$type:ty,ACTIVE) => {
    get_2_ints!($state, $w, $fail, $type, ACTIVE, "OUT OF BOUNDS")
  };
  ($state:ident,$w:ident,$fail:block,$type:ty) => {{
    let (i1, i2) = get_2_ints!($state, $w, $fail, $type, ACTIVE);
    let v2 = $state.current().stack.pop().unwrap();
    let v1 = $state.current().stack.pop().unwrap();
    $state.pool.add_val(v1);
    $state.pool.add_val(v2);
    (i1, i2)
  }};
  ($state:ident,$w:ident,$fail:block) => {
    get_2_ints!($state, $w, $fail, i32)
  };
  ($state:ident,$w:ident$(,$t:tt)*) => {
    get_2_ints!($state, $w, {} $(,$t)*)
  }
}
