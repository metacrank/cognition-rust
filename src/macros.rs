#![allow(unused_imports)]

pub const RED: &[u8] = b"\x1B[31m";
pub const HBLK: &[u8] = b"\x1B[90m";
//pub const COLOR_RESET: &[u8] = b"\x1B[39m";
pub const COLOR_RESET: &[u8] = b"\x1B[0m";

pub const DEFAULT_STACK_SIZE: usize = 24;
pub const DEFAULT_STRING_LENGTH: usize = 24;
pub const DEFAULT_WORD_TABLE_SIZE: usize = 576;

#[macro_export]
macro_rules! bad_value_err {
  () => { panic!("Bad value on stack") };
}

#[macro_export]
macro_rules! default_fprint_error {
  ($e:literal) => {
    print!("Value::fprint(): error: ");
    println!($e);
  }
}
#[macro_export]
macro_rules! fwrite_check_byte {
  ($f:ident,$s:expr,$n:ident) => {
    let n = $n;
    match $f.write($s) {
      Ok(1) => {},
      Ok(_) => { $crate::default_fprint_error!("not all bytes could be written");
                 $crate::default_fprint_error!("wrote {n} bytes"); },
      Err(e) => { $crate::default_fprint_error!("{e}"); },
    }
  }
}

#[macro_export]
macro_rules! fwrite_check {
  ($f:ident,$s:expr) => {
    let s: &[u8] = $s;
    for n in 0..s.len() {
      $crate::fwrite_check_byte!($f, &s[n..n+1], n);
    }
  }
}
// pub(crate) use fwrite_check;

#[macro_export]
macro_rules! fwrite_check_pretty {
  ($f:ident,$s:expr) => {
    let s: &[u8] = $s;
    for n in 0..s.len() {
      match s[n] {
        b'\n' => {
          $crate::fwrite_check_byte!($f, b"\\", n);
          $crate::fwrite_check_byte!($f, b"n", n);
        },
        b'\t' => {
          $crate::fwrite_check_byte!($f, b"\\", n);
          $crate::fwrite_check_byte!($f, b"t", n);
        },
        _ => {
          $crate::fwrite_check_byte!($f, &s[n..n+1], n);
        },
      }
    }
  }
}
// pub(crate) use fwrite_check_pretty;

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
  // handle recursion
  ($state:ident,$n:expr,$fn:ident $(,$fi:ident)*) => {{
    let mut vmacro = build_macro!($state, $n $(,$fi)*);
    let v = $state.pool.get_vfllib($fn);
    vmacro.macro_stack.push($crate::Value::FLLib(v));
    vmacro
  }};
  // reverse and count arguments
  ($state:ident,$n:expr,[] $($fr:ident)*) => {
    build_macro!($state, $n $(,$fr)*)
  };
  ($state:ident,$n:expr,[$fn:ident $($fi:ident)*] $($fr:ident)*) => {
    build_macro!($state, $n + 1, [$($fi)*] $fn $($fr)*)
  }
}
/// add_word!(state: CognitionState, name: &'static str, f1, f2, ..., fn: CognitionFunction)
/// mutates state and inserts a macro word containing f1, f2, ..., fn as vfllibs into state
/// current stack's word_table. If only one CognitionFunction parameter was given, then the
/// resulting vfllib str_word value is derived from 'name'. Otherwise, they are all nameless.
#[macro_export]
macro_rules! add_word {
  ($state:ident,$name:literal,$f:ident) => {
    let mut vmacro = build_macro!($state, 1);

    let mut vfllib = $state.pool.get_vfllib($f);
    vfllib.str_word = Some(String::from($name));

    vmacro.macro_stack.push($crate::Value::FLLib(vfllib));
    $state.current().add_word($crate::Value::Macro(vmacro), std::string::String::from($name));
  };
  ($state:ident,$name:literal$ (,$f:ident)*) => {
    let vmacro = build_macro!($state, 0, [$($f)*]);
    $state.current().add_word($crate::Value::Macro(vmacro), std::string::String::from($name));
  }
}
// pub(crate) use add_word;
