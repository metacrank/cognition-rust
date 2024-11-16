use crate::*;
use crate::math::*;
use std::fmt;
use std::marker::PhantomData;
use ::serde::de::{self, Deserializer, DeserializeSeed, EnumAccess, IgnoredAny, MapAccess, SeqAccess, VariantAccess, Visitor};
use ::serde::ser::{Serialize, Serializer, SerializeMap, SerializeSeq, SerializeStruct};

pub struct Wrap<'a, T: ?Sized>(pub &'a T);

impl<'a, T> Serialize for Wrap<'a, T>
where T: ?Sized + erased_serde::Serialize + 'a,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer,
  {
    erased_serde::serialize(self.0, serializer)
  }
}

pub struct ArcWrap<'a, T: ?Sized>(pub &'a Arc<T>);

impl<'a, T> Serialize for ArcWrap<'a, T>
where T: ?Sized + Serialize + 'a,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer,
  {
    (**self.0).serialize(serializer)
  }
}

pub struct VecWrap<'a, T: Sized>(pub &'a Vec<T>);

impl<'a, T> Serialize for VecWrap<'a, T>
where T: Sized + Serialize + 'a,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer,
  {
    let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
    for e in self.0 {
      seq.serialize_element(e)?;
    }
    seq.end()
  }
}

#[derive(Serialize)]
pub struct Element<K: Serialize,V: Serialize>(K, V);

pub struct MapWrap<'a, K, V>(pub &'a HashMap<K, V>);

impl<K, V> Serialize for MapWrap<'_, K, V>
where
  K: Serialize,
  V: Serialize
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {

    let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
    for (k, v) in self.0.iter() {
      let element = Element(k, v);
      seq.serialize_element(&element)?;
    }
    seq.end()
  }
}

trait GetCustomFromLibraries {
  fn get_fn(&self, key: &str) -> Option<&DeserializeFn<dyn Custom>>;
}

impl GetCustomFromLibraries for ForeignLibraries {
  fn get_fn(&self, key: &str) -> Option<&DeserializeFn<dyn Custom>> {
    for (k, function) in BUILTIN_CUSTOM_DESERIALIZERS.iter() {
      if *k == key { return Some(&function) }
    }
    for library in self.values() {
      if let Some(func) = library.registry.get(key) { return Some(func) }
    }
    None
  }
}

pub struct FnApply<T: ?Sized> {
  deserialize_fn: DeserializeFn<T>
}

impl<'de, T: ?Sized> DeserializeSeed<'de> for FnApply<T> {
  type Value = Box<T>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where D: Deserializer<'de>,
  {
    let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
    (self.deserialize_fn)(&mut erased).map_err(de::Error::custom)
  }
}

pub struct CognitionDeserializeSeed<'s, 'de: 's, T: Sized + CognitionDeserialize<'de>> {
  state: &'s mut CognitionState,
  cognition_type: PhantomData<&'de T>,
}

impl<'s, 'de: 's, T> CognitionDeserializeSeed<'s, 'de, T>
where T: Sized + CognitionDeserialize<'de>
{
  pub fn new(state: &'s mut CognitionState) -> Self {
    Self{ state, cognition_type: PhantomData }
  }
}

impl<'s, 'de: 's, T> DeserializeSeed<'de> for CognitionDeserializeSeed<'s, 'de, T>
where T: Sized + CognitionDeserialize<'de>
{
  type Value = T;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where D: Deserializer<'de>,
  {
    <T>::cognition_deserialize(deserializer, self.state)
  }
}

pub struct CustomVisitor<'r> {
  libraries: &'r Option<ForeignLibraries>
}

impl<'r, 'de: 'r> Visitor<'de> for CustomVisitor<'r> {
  type Value = Box<dyn Custom>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "dyn Custom")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where A: MapAccess<'de>,
  {
    let map_lookup = CustomMapLookupVisitor { libraries: self.libraries };
    let deserialize_fn = match map.next_key_seed(map_lookup)? {
      Some(deserialize_fn) => deserialize_fn,
      None => {
        return Err(de::Error::custom(format_args!("expected dyn Custom")));
      }
    };
    map.next_value_seed(FnApply { deserialize_fn })
  }
}

pub struct CustomMapLookupVisitor<'r> {
  pub libraries: &'r Option<ForeignLibraries>
}

impl<'r> Copy for CustomMapLookupVisitor<'r> {}

impl<'r> Clone for CustomMapLookupVisitor<'r> {
  fn clone(&self) -> Self { *self }
}

impl<'r, 'de: 'r> Visitor<'de> for CustomMapLookupVisitor<'r> {
  type Value = DeserializeFn<dyn Custom>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "dyn Custom")
  }

  fn visit_str<E>(self, key: &str) -> Result<Self::Value, E>
  where E: de::Error,
  {
    for (k, func) in BUILTIN_CUSTOM_DESERIALIZERS.iter() {
      if *k == key { return Ok(*func) }
    }

    if let Some(ref libs) = self.libraries {
      if let Some(value) = libs.get_fn(key) {
        return Ok(*value);
      }
    }
    Err(de::Error::custom(format_args!("unknown Custom type")))
  }
}

impl<'r, 'de: 'r> DeserializeSeed<'de> for CustomMapLookupVisitor<'r> {
  type Value = DeserializeFn<dyn Custom>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where D: Deserializer<'de>,
  {
    deserializer.deserialize_str(self)
  }
}

pub fn serialize_custom<S, T>(serializer: S, variant: &'static str, concrete: &T) -> Result<S::Ok, S::Error>
where
  S: Serializer,
  T: ?Sized + erased_serde::Serialize,
{
  let mut ser = serializer.serialize_map(Some(1))?;
  ser.serialize_entry(variant, &Wrap(concrete))?;
  ser.end()
}

pub fn deserialize_custom<'de, D>(deserializer: D, state: &CognitionState) -> Result<Box<dyn Custom>, D::Error>
where D: Deserializer<'de>,
{
  let libraries = &state.fllibs;
  let visitor = CustomVisitor{ libraries };
  deserializer.deserialize_map(visitor)
}

pub fn deserialize_library<'de, D>(deserializer: D, state: &mut CognitionState) -> Result<Library, D::Error>
where D: Deserializer<'de>,
{
  #[derive(Deserialize)]
  #[serde(field_identifier, rename_all = "snake_case")]
  enum Field { LibName }

  struct LibraryVisitor<'s> {
    state: &'s mut CognitionState
  }

  impl<'s, 'de: 's> Visitor<'de> for LibraryVisitor<'s> {
    type Value = Library;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "Library")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
      V: SeqAccess<'de>,
    {
      let lib_name: &'de str = seq.next_element()?
        .ok_or_else(|| de::Error::invalid_length(0, &self))?;

      let fllibs = self.state.fllibs.take();
      if let Some(ref libs) = fllibs {
        if let Some(library) = libs.get(lib_name) {
          let cloned_library = library.library.clone();
          self.state.fllibs = fllibs;
          return Ok(cloned_library)
        }
      }
      self.state.fllibs = fllibs;
      Err(de::Error::custom(format_args!("unknown library {}", lib_name)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de>,
    {
      let mut lib_name: Option<&'de str> = None;
      while let Some(key) = map.next_key()? {
        match key {
          Field::LibName => {
            if lib_name.is_some() {
              return Err(de::Error::duplicate_field("lib_name"));
            }
            lib_name = Some(map.next_value()?);
          }
        }
      }
      let lib_name = lib_name.ok_or_else(|| de::Error::missing_field("lib_name"))?;

      let fllibs = self.state.fllibs.take();
      if let Some(ref libs) = fllibs {
        if let Some(library) = libs.get(lib_name) {
          let cloned_library = library.library.clone();
          self.state.fllibs = fllibs;
          return Ok(cloned_library)
        }
      }
      self.state.fllibs = fllibs;
      Err(de::Error::custom(format_args!("unknown library {}", lib_name)))
    }
  }

  let visitor = LibraryVisitor{ state };
  const FIELDS: &[&str] = &["lib_name"];
  deserializer.deserialize_struct("Library", FIELDS, visitor)
}

impl Serialize for dyn Custom {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let name = <Self as Custom>::custom_type_name(self);
    serialize_custom(serializer, name, self)
  }
}

impl Serialize for FLLibLibrary {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("Library", 2)?;
    ser.serialize_field("lib_name", &self.lib_name)?;
    ser.serialize_field("lib_path", &self.lib_path)?;
    ser.end()
  }
}

impl Serialize for VFLLib {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("VFLLib", 3)?;
    ser.serialize_field("str_word", &self.str_word)?;
    let lib = match self.library {
      Some(ref library) => Some(ArcWrap(library)),
      None => None
    };
    ser.serialize_field("library", &lib)?;
    ser.serialize_field("key", &self.key)?;
    ser.end()
  }
}

impl Serialize for Value {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    match *self {
      Value::Word(ref vw)  => serializer.serialize_newtype_variant("Value", 0, "Word", vw),
      Value::Stack(ref vs) => serializer.serialize_newtype_variant("Value", 1, "Stack", vs),
      Value::Macro(ref vm) => serializer.serialize_newtype_variant("Value", 2, "Macro", vm),
      Value::Error(ref ve) => serializer.serialize_newtype_variant("Value", 3, "Error", ve),
      Value::FLLib(ref vf) => serializer.serialize_newtype_variant("Value", 4, "FLLib", vf),
      Value::Custom(ref vc) => serializer.serialize_newtype_variant("Value", 5, "Custom", vc),
      Value::Control(ref vc) => serializer.serialize_newtype_variant("Value", 6, "Control", vc),
    }
  }
}

impl Serialize for VMacro {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("VMacro", 1)?;
    ser.serialize_field("macro_stack", &VecWrap(&self.macro_stack))?;
    ser.end()
  }
}

struct WordTableWrap<'a>(pub &'a WordTable);

impl Serialize for WordTableWrap<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for (k, v) in self.0 {
      map.serialize_entry(k, &ArcWrap(v))?;
    }
    map.end()
  }
}

struct FamilyWrap<'a>(pub &'a Family);

impl Serialize for FamilyWrap<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
    for v in self.0 {
      seq.serialize_element(&ArcWrap(v))?;
    }
    seq.end()
  }
}

impl Serialize for Container {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("Container", 12)?;
    ser.serialize_field("stack", &self.stack)?;
    ser.serialize_field("err_stack", &self.err_stack)?;
    ser.serialize_field("cranks", &self.cranks)?;
    ser.serialize_field("math", &self.math)?;
    ser.serialize_field("faliases", &self.faliases)?;
    ser.serialize_field("delims", &self.delims)?;
    ser.serialize_field("ignored", &self.ignored)?;
    ser.serialize_field("singlets", &self.singlets)?;
    ser.serialize_field("dflag", &self.dflag)?;
    ser.serialize_field("iflag", &self.iflag)?;
    ser.serialize_field("sflag", &self.sflag)?;
    let word_table = match self.word_table {
      Some(ref wtable) => Some(WordTableWrap(wtable)),
      None => None
    };
    ser.serialize_field("word_table", &word_table)?;
    ser.end()
  }
}

impl Serialize for VStack {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("VStack", 1)?;
    ser.serialize_field("container", &self.container)?;
    ser.end()
  }
}

impl Serialize for Op {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    match *self {
      Op::Unary(ref u)  => serializer.serialize_newtype_variant("Op", 0, "Unary", &MapWrap(u)),
      Op::Binary(ref b)  => serializer.serialize_newtype_variant("Op", 0, "Binary", &MapWrap(b)),
      Op::Str(ref s)  => serializer.serialize_newtype_variant("Op", 0, "Str", s),
      Op::Custom(ref c)  => serializer.serialize_newtype_variant("Op", 0, "Custom", &MapWrap(c)),
    }
  }
}

impl Serialize for Math {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("Math", 10)?;
    ser.serialize_field("base", &self.base)?;
    ser.serialize_field("digits", &self.digits)?;
    ser.serialize_field("d_idx", &MapWrap(&self.d_idx))?;
    ser.serialize_field("mul", &MapWrap(&self.mul))?;
    ser.serialize_field("ops_table", &self.ops_table)?;
    ser.serialize_field("negc", &self.negc)?;
    ser.serialize_field("radix", &self.radix)?;
    ser.serialize_field("delim", &self.delim)?;
    ser.serialize_field("meta_radix", &self.meta_radix)?;
    ser.serialize_field("meta_delim", &self.meta_delim)?;
    ser.end()
  }
}

pub trait CognitionDeserialize<'de> {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized;
}

impl<'de> CognitionDeserialize<'de> for Box<dyn Custom> {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    deserialize_custom(deserializer, state)
  }
}

impl<'de> CognitionDeserialize<'de> for Library {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    deserialize_library(deserializer, state)
  }
}

impl<'de> CognitionDeserialize<'de> for Value {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    #[derive(Deserialize)]
    enum Field { Word, Stack, Macro, Error, FLLib, Custom, Control }

    struct CognitionVisitor<'s> {
      state: &'s mut CognitionState
    }

    impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
      type Value = Value;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "enum Value")
      }

      fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
      where
        A: EnumAccess<'de>,
      {
        let (variant_name, variant_data): (Field, _) = data.variant()?;

        match variant_name {
          Field::Word => Ok(Value::Word(variant_data.newtype_variant()?)),
          Field::Stack => {
            let seed = CognitionDeserializeSeed::<Box<VStack>>::new(self.state);
            Ok(Value::Stack(variant_data.newtype_variant_seed(seed)?))
          },
          Field::Macro => {
            let seed = CognitionDeserializeSeed::<Box<VMacro>>::new(self.state);
            Ok(Value::Macro(variant_data.newtype_variant_seed(seed)?))
          },
          Field::Error => Ok(Value::Error(variant_data.newtype_variant()?)),
          Field::FLLib => {
            let seed = CognitionDeserializeSeed::<Box<VFLLib>>::new(self.state);
            Ok(Value::FLLib(variant_data.newtype_variant_seed(seed)?))
          },
          Field::Custom => {
            let seed = CognitionDeserializeSeed::<VCustom>::new(self.state);
            Ok(Value::Custom(variant_data.newtype_variant_seed(seed)?))
          },
          Field::Control => Ok(Value::Control(variant_data.newtype_variant()?)),
        }
      }
    }

    let visitor = CognitionVisitor{ state };
    const FIELDS: &[&str] = &["Word", "Stack", "Macro", "Error", "FLLib", "Custom", "Control"];
    deserializer.deserialize_enum("Value", FIELDS, visitor)
  }
}

impl<'de> CognitionDeserialize<'de> for Op {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    #[derive(Deserialize)]
    enum Field { Unary, Binary, Str, Custom }

    struct CognitionVisitor<'s> {
      state: &'s mut CognitionState
    }

    impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
      type Value = Op;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "enum Op")
      }

      fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
      where
        A: EnumAccess<'de>,
      {
        let (variant_name, variant_data): (Field, _) = data.variant()?;

        match variant_name {
          Field::Unary => {
            let seed = CognitionDeserializeSeed::<UnaryOp>::new(self.state);
            Ok(Op::Unary(variant_data.newtype_variant_seed(seed)?))
          },
          Field::Binary => {
            let seed = CognitionDeserializeSeed::<BinaryOp>::new(self.state);
            Ok(Op::Binary(variant_data.newtype_variant_seed(seed)?))
          },
          Field::Str => Ok(Op::Str(variant_data.newtype_variant()?)),
          Field::Custom => {
            let seed = CognitionDeserializeSeed::<CustomOp>::new(self.state);
            Ok(Op::Custom(variant_data.newtype_variant_seed(seed)?))
          },
        }
      }
    }

    let visitor = CognitionVisitor{ state };
    const FIELDS: &[&str] = &["Unary", "Binary", "Str", "Custom"];
    deserializer.deserialize_enum("Op", FIELDS, visitor)
  }
}

impl<'de> CognitionDeserialize<'de> for WordDef {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    let inner = <Value>::cognition_deserialize(deserializer, state)?;
    Ok(state.pool.get_word_def(inner))
  }
}

macro_rules! impl_cognition_deserialize {
  ($type:ty,$deserializer:tt,$state:tt,$block:block) => {
    impl<'de> CognitionDeserialize<'de> for $type {
      fn cognition_deserialize<D>($deserializer: D, _: &mut CognitionState) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
        Self: Sized,
      {
        $block
      }
    }
  };
  ($($type:ty,$deserializer:tt,$state:tt,$block:block);*) => {
    $( impl_cognition_deserialize_for_deserialize!{ $type, $deserializer, $state, $block } )*
  }
}

macro_rules! impl_cognition_deserialize_for_deserialize {
  ($($type:ty),*) => {
    $( impl_cognition_deserialize!{ $type, deserializer, _state, { <$type>::deserialize(deserializer) } } )*
  }
}

macro_rules! impl_cognition_deserialize_struct {
  ($type:ty,$typename:literal,$state:tt,$from:block $([$capital:tt,$lower:tt,$name:literal,$subtype:ty]),*) => {
    impl <'de> CognitionDeserialize<'de> for $type {
      fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
        Self: Sized,
      {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { $($capital),* }

        struct CognitionVisitor<'s> {
          state: &'s mut CognitionState
        }

        impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
          type Value = $type;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "struct {}", $typename)
          }

          fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
          where
            V: SeqAccess<'de>,
          {
            $(
              let cogseed = CognitionDeserializeSeed::<$subtype>::new(self.state);
              let $lower = seq.next_element_seed(cogseed)?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?; // ${index()} is still unstable
            )*
            let $state = self.state;
            $from
          }

          fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
          where A: MapAccess<'de>,
          {
            $(
              let mut $lower = None;
            )*
            while let Some(key) = map.next_key()? {
              match key {
                $(
                  Field::$capital => {
                    if $lower.is_some() {
                      return Err(de::Error::duplicate_field($name));
                    }
                    let cogseed = CognitionDeserializeSeed::<$subtype>::new(self.state);
                    $lower = Some(map.next_value_seed(cogseed)?);
                  }
                )*
              }
            }
            $(
              let $lower = $lower.ok_or_else(|| de::Error::missing_field($name))?;
            )*
            let $state = self.state;
            $from
          }
        }

        let visitor = CognitionVisitor{ state };
        const FIELDS: &[&str] = &[$($name),*];
        deserializer.deserialize_struct($typename, FIELDS, visitor)
      }
    }
  };
  ($type:ty,$typename:literal,$from:block $([$capital:tt,$lower:tt,$name:literal,$subtype:ty])*) => {
    impl_cognition_deserialize_struct!{ $type, $typename, _state, $from $([$capital,$lower,$name,$subtype])* }
  }
}

macro_rules! impl_cognition_deserialize_option {
  ($type:ty,$typename:literal) => {
    impl<'de> CognitionDeserialize<'de> for Option<$type> {
      fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
        Self: Sized,
      {
        struct CognitionVisitor<'s> {
          state: &'s mut CognitionState
        }
        impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
          type Value = Option<$type>;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, $typename)
          }
          fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
          }
          fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
          where
            D: Deserializer<'de>,
          {
            let library = <$type>::cognition_deserialize(deserializer, self.state)?;
            Ok(Some(library))
          }
        }

        let visitor = CognitionVisitor{ state };
        deserializer.deserialize_option(visitor)
      }
    }
  }
}

macro_rules! impl_cognition_deserialize_seq {
  ($type:ty,$state1:tt,$state2:tt,$stack:tt,$len:tt,$getstack:block,$elseexpr:block,$expecting:literal) => {
    impl<'de> CognitionDeserialize<'de> for Vec<$type> {
      fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
        Self: Sized,
      {
        struct CognitionVisitor<'s> {
          state: &'s mut CognitionState
        }
        impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
          type Value = Vec<$type>;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str($expecting)
          }

          fn visit_seq<V>(mut self, mut seq: V) -> Result<Self::Value, V::Error>
          where
            V: SeqAccess<'de>,
          {
            let $len = seq.size_hint().unwrap_or(DEFAULT_STACK_SIZE);
            let $state1 = &mut self.state;
            let mut $stack = $getstack;
            loop {
              let cogseed = CognitionDeserializeSeed::<$type>::new(self.state);
              let Ok(element) = seq.next_element_seed(cogseed) else {
                let $state2 = &mut self.state;
                $elseexpr;
                return Err(de::Error::custom(format_args!("invalid element for {}", $expecting)))
              };
              let Some(element) = element else { break };
              $stack.push(element);
            }
            Ok($stack)
          }
        }

        let visitor = CognitionVisitor{ state };
        deserializer.deserialize_seq(visitor)
      }
    }
  }
}

macro_rules! impl_cognition_deserialize_table {
  ($type:ty,$state1:tt,$state2:tt,$map:tt,$len:tt,$getmap:block,$elseexpr:block,$expecting:literal,$default_len:expr) => {
    impl<'de> CognitionDeserialize<'de> for HashMap<String, $type> {
      fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
        Self: Sized,
      {
        struct CognitionVisitor<'s> {
          state: &'s mut CognitionState
        }
        impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
          type Value = HashMap<String, $type>;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str($expecting)
          }

          fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
          where
            V: MapAccess<'de>,
          {
            let $len = map.size_hint().unwrap_or($default_len);
            let $state1 = &mut self.state;
            let mut $map = $getmap;

            while let Some(key) = map.next_key()? {
              let cogseed = CognitionDeserializeSeed::<$type>::new(self.state);
              let Ok(value) = map.next_value_seed(cogseed) else {
                let $state2 = &mut self.state;
                $elseexpr;
                return Err(de::Error::custom(format_args!("invalid element for {}", $expecting)))
              };
              $map.insert(key, value);
            }
            Ok($map)
          }
        }

        let visitor = CognitionVisitor{ state };
        deserializer.deserialize_map(visitor)
      }
    }
  }
}

macro_rules! impl_cognition_deserialize_map_as_vec {
  ($type:ty,$subtype1:ty,$subtype2:ty,$state1:tt,$state2:tt,$map:tt,$getmap:block,$elseexpr:block,$expecting:literal,$default_len:expr) => {
    impl<'de> CognitionDeserialize<'de> for $type {
      fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
        Self: Sized,
      {
        struct CognitionVisitor<'s> {
          state: &'s mut CognitionState
        }
        impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
          type Value = $type;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str($expecting)
          }

          fn visit_seq<V>(mut self, mut seq: V) -> Result<Self::Value, V::Error>
          where
            V: SeqAccess<'de>,
          {
            let $state1 = &mut self.state;
            let mut $map = $getmap;
            loop {
              let cogseed = CognitionDeserializeSeed::<($subtype1, $subtype2)>::new(self.state);
              let Ok(element) = seq.next_element_seed(cogseed) else {
                let $state2 = &mut self.state;
                $elseexpr;
                return Err(de::Error::custom(format_args!("invalid element for {}", $expecting)))
              };
              let Some(element) = element else { break };
              $map.insert(element.0, element.1);
            }
            Ok($map)
          }
        }

        let visitor = CognitionVisitor{ state };
        deserializer.deserialize_seq(visitor)
      }
    }
  }
}

impl_cognition_deserialize_for_deserialize! {
  bool, u32, i32, char, String,
  (i32, i32), (char, i32), (char, char),
  ((i32, i32), (i32, i32)),
  ((char, char), (i32, i32)),
  Digit, (Digit, Digit),
  ((Digit, Digit), (Digit, Digit)),
  Operand, (Operand, Operand),
  Vec<Digit>, Vec<char>,
  Option<char>,
  Option<String>,
  Option<Cranks>,
  Option<Faliases>,
  Option<Parser>
  //Pool
}

impl_cognition_deserialize_option!{ Library, "Option<Library>" }
impl_cognition_deserialize_option!{ Stack, "Option<Stack>" }
impl_cognition_deserialize_option!{ WordTable, "Option<WordTable>" }
impl_cognition_deserialize_option!{ Math, "Option<Math>" }

impl_cognition_deserialize_table! {
  WordDef, state, state, map, len,
  { state.pool.get_word_table(len) },
  { state.pool.add_word_table(map); },
  "WordTable", DEFAULT_WORD_TABLE_SIZE
}

impl_cognition_deserialize_table! {
  Op, _state, state, map, len,
  { OpsTable::with_capacity(len) },
  {
    for (k, v) in map.into_iter() {
      state.pool.add_string(k);
      state.pool.add_op(v);
    }
  },
  "OpsTable", DEFAULT_OPS_TABLE_SIZE
}

impl_cognition_deserialize_map_as_vec! {
  UnaryOp, Digit, Digit, state, state, map,
  { state.pool.get_un_op() },
  { state.pool.add_un_op(map); },
  "UnaryOp", DEFAULT_OP_SIZE
}

impl_cognition_deserialize_map_as_vec! {
  BinaryOp, (Digit, Digit), (Digit, Digit), state, state, map,
  { state.pool.get_bin_op() },
  { state.pool.add_bin_op(map); },
  "BinaryOp", DEFAULT_OP_SIZE
}

impl_cognition_deserialize_map_as_vec! {
  CustomOp, Operand, Operand, state, state, map,
  { state.pool.get_custom_op() },
  { state.pool.add_custom_op(map); },
  "CustomOp", DEFAULT_OP_SIZE
}

impl_cognition_deserialize_map_as_vec! {
  HashMap<char, i32>, char, i32, _state, _state, map,
  { HashMap::new() }, { },
  "HashMap<char, i32>", 0
}

impl_cognition_deserialize_map_as_vec! {
  HashMap<(char, char), (i32, i32)>, (char, char), (i32, i32), _state, _state, map,
  { HashMap::new() }, { },
  "HashMap<(char, char), (i32, i32)>", 0
}

impl_cognition_deserialize_seq!{
  Value, state, state, stack, len,
  { state.pool.get_stack(len) },
  { state.pool.add_stack(stack) },
  "Stack"
}
impl_cognition_deserialize_seq!{
  Stack, _state, state, metastack, len,
  { Vec::<Stack>::with_capacity(len) },
  { for s in metastack.into_iter() { state.pool.add_stack(s); } },
  "Vec<Stack>"
}
impl_cognition_deserialize_seq!{
  WordDef, state, state, family, _len,
  { state.pool.get_family() },
  { state.pool.add_family(family) },
  "Family"
}

impl_cognition_deserialize_struct! {
  VCustom, "VCustom", { Ok(VCustom::with_custom(custom)) }
  [Custom, custom, "custom", Box<dyn Custom>]
}

impl_cognition_deserialize_struct! {
  Box<VMacro>, "VMacro", state, {
    let mut vmacro = state.pool.get_vmacro(0);
    let mut mutable_macro_stack = macro_stack;
    std::mem::swap(&mut vmacro.macro_stack, &mut mutable_macro_stack);
    state.pool.add_stack(mutable_macro_stack);
    Ok(vmacro)
  }
  [MacroStack, macro_stack, "macro_stack", Stack]
}

impl_cognition_deserialize_struct! {
  Container, "Container", _state, {
    let mut container = Container::with_stack(stack);
    container.err_stack = err_stack;
    container.cranks = cranks;
    container.math = math;
    container.faliases = faliases;
    container.delims = delims;
    container.ignored = ignored;
    container.singlets = singlets;
    container.dflag = dflag;
    container.iflag = iflag;
    container.sflag = sflag;
    container.word_table = word_table;
    Ok(container)
  }
  [Stack, stack, "stack", Stack],
  [ErrStack, err_stack, "err_stack", Option<Stack>],
  [Cranks, cranks, "cranks", Option<Cranks>],
  [Math, math, "math", Option<Math>],
  [Faliases, faliases, "faliases", Option<Faliases>],
  [Delims, delims, "delims", Option<String>],
  [Ignored, ignored, "ignored", Option<String>],
  [Singlets, singlets, "singlets", Option<String>],
  [Dflag, dflag, "dflag", bool],
  [Iflag, iflag, "iflag", bool],
  [Sflag, sflag, "sflag", bool],
  [WordTable, word_table, "word_table", Option<WordTable>]
}

impl_cognition_deserialize_struct! {
  Math, "Math", _state, {
    let mut math = Math::new();
    math.base = base;
    math.digits = digits;
    math.d_idx = d_idx;
    math.mul = mul;
    math.ops_table = ops_table;
    math.negc = negc;
    math.radix = radix;
    math.delim = delim;
    math.meta_radix = meta_radix;
    math.meta_delim = meta_delim;
    Ok(math)
  }
  [Base, base, "base", i32],
  [Digits, digits, "digits", Vec<char>],
  [DIdx, d_idx, "d_idx", HashMap<char, i32>],
  [Mul, mul, "mul", HashMap<(char, char), (i32, i32)>],
  [OpsTable, ops_table, "ops_table", OpsTable],
  [Negc, negc, "negc", Option<char>],
  [Radix, radix, "radix", Option<char>],
  [Delim, delim, "delim", Option<char>],
  [MetaRadix, meta_radix, "meta_radix", Option<char>],
  [MetaDelim, meta_delim, "meta_delim", Option<char>]
}

impl_cognition_deserialize_struct! {
  Box<VStack>, "VStack", state, {
    let mut vstack = state.pool.get_vstack(0);
    let _ = std::mem::replace(&mut vstack.container, container);
    Ok(vstack)
  }
  [Container, container, "container", Container]
}

impl_cognition_deserialize_struct! {
  Box<VFLLib>, "VFLLib", state, {
    let func = if let Some(ref library) = library {
      let lib_name: &str = &library.lib_name;
      match state.fllibs {
        Some(ref libs) => {
          match libs.get(lib_name) {
            Some(foreign_lib) => {
              match foreign_lib.functions.get(key as usize) {
                Some(func) => func.clone(),
                None => return Err(de::Error::custom(format_args!("invalid fllib index: {}", key)))
              }
            },
            None => return Err(de::Error::custom(format_args!("unknown library: {}", lib_name)))
          }
        },
        None => return Err(de::Error::custom(format_args!("unknown library: {}", lib_name)))
      }
    } else {
      match state.builtins.get(key as usize) {
        Some(func) => func.clone(),
        None => return Err(de::Error::custom(format_args!("invalid builtin index: {}", key)))
      }
    };
    let mut vfllib = state.pool.get_vfllib(func);
    vfllib.str_word = str_word;
    vfllib.library = library;
    vfllib.key = key;
    Ok(vfllib)
  }
  [StrWord, str_word, "str_word", Option<String>],
  [Library, library, "library", Option<Library>],
  [Key, key, "key", u32]
}

#[derive(Serialize, Deserialize)]
pub struct LibraryDescriptor<'a> {
  lib_path: &'a str,
  lib_name: &'a str,
}

struct LibrarySeed;

impl<'s, 'de: 's> DeserializeSeed<'de> for LibrarySeed {
  type Value = String;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where D: Deserializer<'de>,
  {
    struct Vis;
    impl<'de> Visitor<'de> for Vis {
      type Value = String;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("str")
      }

      fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
      where E: de::Error
      {
        Ok(v)
      }
    }
    deserializer.deserialize_string(Vis)
  }
}

pub fn serde_desc_fllibs<S>(serializer: S, libs: &Option<ForeignLibraries>) -> Result<S::Ok, S::Error>
where S: Serializer
{
  let len = match libs {
    Some(ref libs) => Some(libs.len()),
    None => Some(0)
  };
  let mut map = serializer.serialize_map(len)?;
  if let Some(ref libs) = libs {
    for (name, lib) in libs.iter() {
      map.serialize_entry(name, &lib.library.lib_path)?;
    }
  }
  map.end()
}

pub fn serde_load_fllibs<'de, D>(deserializer: D, state: &mut CognitionState) -> Result<Option<&'static str>, D::Error>
where D: Deserializer<'de>
{
  struct CognitionVisitor<'s> {
    state: &'s mut CognitionState
  }
  impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
    type Value = Option<&'static str>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("Foreign Library Descriptor Map")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where V: MapAccess<'de>,
    {
      while let Some(key) = map.next_key()? {
        let value = map.next_value_seed(LibrarySeed)?;
        let option = unsafe { self.state.load_fllib(&key, &value) };
        self.state.pool.add_string(key);
        self.state.pool.add_string(value);
        if let Some(e) = option {
          return Ok(Some(e))
        }
      }
      Ok(None)
    }
  }

  let visitor = CognitionVisitor{ state };
  deserializer.deserialize_map(visitor)
}

struct ForeignLibrariesWrapper<'a>(pub &'a Option<ForeignLibraries>);

impl Serialize for ForeignLibrariesWrapper<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    serde_desc_fllibs(serializer, self.0)
  }
}

impl Serialize for CognitionState {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    let mut ser = serializer.serialize_struct("CognitionState", 8)?;
    ser.serialize_field("chroots", &self.chroots)?;
    ser.serialize_field("stack", &self.stack)?;
    ser.serialize_field("family", &FamilyWrap(&self.family))?;
    ser.serialize_field("parser", &self.parser)?;
    ser.serialize_field("exited", &self.exited)?;
    ser.serialize_field("exit_code", &self.exit_code)?;
    ser.serialize_field("args", &self.args)?;
    ser.serialize_field("fllibs", &ForeignLibrariesWrapper(&self.fllibs))?;
    //ser.serialize_field("pool", &self.pool)?;
    ser.end()
  }
}

struct ForeignLibrariesRepr;

impl<'de> CognitionDeserialize<'de> for ForeignLibrariesRepr {
  fn cognition_deserialize<D>(deserializer: D, state: &mut CognitionState) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
    Self: Sized,
  {
    match serde_load_fllibs(deserializer, state)? {
      Some(e) => Err(de::Error::custom(format_args!("{}", e))),
      None => Ok(ForeignLibrariesRepr)
    }
  }
}

pub fn cogstate_init() -> CognitionState {
  let mut state = CognitionState::new(Stack::with_capacity(1));
  state.stack.push(Value::Stack(Box::new(VStack::with_capacity(0))));
  crate::builtins::add_builtins(&mut state);
  state
}

pub enum TripleErr<'de, D, F>
where
  D: Deserializer<'de>,
  F: Deserializer<'de>
{
  D(D::Error),
  F(F::Error),
  S(&'static str)
}

impl<'de, D, F> fmt::Display for TripleErr<'de, D, F>
where
  D: Deserializer<'de>,
  F: Deserializer<'de>
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TripleErr::D(e) => e.fmt(f),
      TripleErr::F(e) => e.fmt(f),
      TripleErr::S(s) => write!(f, "{}", s)
    }
  }
}

macro_rules! deserialize_cognition_state_from_state_macro {
  ($deserializer:ident,$state:ident,$seq:tt,$map:tt,$fllibs:tt,$selfref:tt,$seqblock:block,$mapblock:block) => {{
    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum Field {
      Chroots,
      Stack,
      Family,
      Parser,
      Exited,
      ExitCode,
      Args,
      Fllibs,
      //Pool
    }

    struct CognitionVisitor<'s> {
      state: &'s mut CognitionState
    }

    impl<'s, 'de: 's> Visitor<'de> for CognitionVisitor<'s> {
      type Value = Void;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "struct CognitionState")
      }

      fn visit_seq<V>(mut self, mut $seq: V) -> Result<Self::Value, V::Error>
      where
        V: SeqAccess<'de>,
      {
        let cogseed = CognitionDeserializeSeed::<Vec<Stack>>::new(self.state);
        let chroots = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let cogseed = CognitionDeserializeSeed::<Stack>::new(self.state);
        let stack = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let cogseed = CognitionDeserializeSeed::<Family>::new(self.state);
        let family = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        let cogseed = CognitionDeserializeSeed::<Option<Parser>>::new(self.state);
        let parser = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(3, &self))?;
        let cogseed = CognitionDeserializeSeed::<bool>::new(self.state);
        let exited = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(4, &self))?;
        let cogseed = CognitionDeserializeSeed::<Option<String>>::new(self.state);
        let exit_code = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(5, &self))?;
        let cogseed = CognitionDeserializeSeed::<Stack>::new(self.state);
        let args = $seq.next_element_seed(cogseed)?
          .ok_or_else(|| de::Error::invalid_length(6, &self))?;

        let $selfref = &mut self;
        $seqblock
        //let cogseed = CognitionDeserializeSeed::<Pool>::new(self.state);
        //let pool = $seq.next_element_seed(cogseed)?
        //  .ok_or_else(|| de::Error::invalid_length(8, &self))?;

        self.state.chroots = chroots;
        self.state.stack = stack;
        self.state.family = family;
        self.state.parser = parser;
        self.state.exited = exited;
        self.state.exit_code = exit_code;
        self.state.args = args;
        //self.state.pool = pool;
        Ok(Void{})
      }

      fn visit_map<A>(mut self, mut $map: A) -> Result<Self::Value, A::Error>
      where A: MapAccess<'de>,
      {
        let mut chroots = None;
        let mut stack = None;
        let mut family = None;
        let mut parser = None;
        let mut exited = None;
        let mut exit_code = None;
        let mut args = None;
        //let pool = None;

        let mut $fllibs = false;

        while let Some(key) = $map.next_key()? {
          match key {
            Field::Chroots => {
              if chroots.is_some() {
                return Err(de::Error::duplicate_field("chroots"));
              }
              let cogseed = CognitionDeserializeSeed::<Vec<Stack>>::new(self.state);
              chroots = Some($map.next_value_seed(cogseed)?);
            },
            Field::Stack => {
              if stack.is_some() {
                return Err(de::Error::duplicate_field("stack"));
              }
              let cogseed = CognitionDeserializeSeed::<Stack>::new(self.state);
              stack = Some($map.next_value_seed(cogseed)?);
            },
            Field::Family => {
              if family.is_some() {
                return Err(de::Error::duplicate_field("family"));
              }
              let cogseed = CognitionDeserializeSeed::<Family>::new(self.state);
              family = Some($map.next_value_seed(cogseed)?);
            },
            Field::Parser => {
              if parser.is_some() {
                return Err(de::Error::duplicate_field("parser"));
              }
              let cogseed = CognitionDeserializeSeed::<Option<Parser>>::new(self.state);
              parser = Some($map.next_value_seed(cogseed)?);
            },
            Field::Exited => {
              if exited.is_some() {
                return Err(de::Error::duplicate_field("exited"));
              }
              let cogseed = CognitionDeserializeSeed::<bool>::new(self.state);
              exited = Some($map.next_value_seed(cogseed)?);
            },
            Field::ExitCode => {
              if exit_code.is_some() {
                return Err(de::Error::duplicate_field("exit_code"));
              }
              let cogseed = CognitionDeserializeSeed::<Option<String>>::new(self.state);
              exit_code = Some($map.next_value_seed(cogseed)?);
            },
            Field::Args => {
              if args.is_some() {
                return Err(de::Error::duplicate_field("args"));
              }
              let cogseed = CognitionDeserializeSeed::<Stack>::new(self.state);
              args = Some($map.next_value_seed(cogseed)?);
            },
            Field::Fllibs => {
              if $fllibs {
                return Err(de::Error::duplicate_field("fllibs"));
              }
              let $selfref = &mut self;
              $mapblock
            },
            //Field::Pool => {
            //  if pool.is_some() {
            //    return Err(de::Error::duplicate_field("pool"));
            //  }
            //  let cogseed = CognitionDeserializeSeed::<Pool>::new(self.state);
            //  pool = Some($map.next_value_seed(cogseed)?);
            //},
          }
        }
        let chroots = chroots.ok_or_else(|| de::Error::missing_field("chroots"))?;
        let stack = stack.ok_or_else(|| de::Error::missing_field("stack"))?;
        let family = family.ok_or_else(|| de::Error::missing_field("family"))?;
        let parser = parser.ok_or_else(|| de::Error::missing_field("parser"))?;
        let exited = exited.ok_or_else(|| de::Error::missing_field("exited"))?;
        let exit_code = exit_code.ok_or_else(|| de::Error::missing_field("exit_code"))?;
        let args = args.ok_or_else(|| de::Error::missing_field("args"))?;
        //let pool = pool.ok_or_else(|| de::Error::missing_field("pool"))?;
        if !$fllibs { return Err(de::Error::missing_field("fllibs")) }

        self.state.chroots = chroots;
        self.state.stack = stack;
        self.state.family = family;
        self.state.parser = parser;
        self.state.exited = exited;
        self.state.exit_code = exit_code;
        self.state.args = args;
        //self.state.pool = pool;
        Ok(Void{})
      }
    }

    let visitor = CognitionVisitor{ state: $state };
    const FIELDS: &[&str] = &[
      "chroots",
      "stack",
      "family",
      "parser",
      "exited",
      "exit_code",
      "args",
      "fllibs",
      //"pool"
    ];
    let _ = $deserializer.deserialize_struct("CognitionState", FIELDS, visitor)?;

    Ok(())
  }}
}

pub fn deserialize_cognition_state_from_state<'de, D>(deserializer: D, state: &mut CognitionState, ignore_fllibs: bool) -> Result<(), D::Error>
where D: Deserializer<'de>
{
  if ignore_fllibs {
    deserialize_cognition_state_from_state_macro!(deserializer,state,seq,map,fllibs,_selfref, {
      let _ = seq.next_element::<IgnoredAny>()?
        .ok_or_else(|| de::Error::invalid_length(7, _selfref))?;
    }, {
      let _ = map.next_value::<IgnoredAny>()?;
      fllibs = true;
    })
  } else {
    deserialize_cognition_state_from_state_macro!(deserializer,state,seq,map,fllibs,selfref, {
      let cogseed = CognitionDeserializeSeed::<ForeignLibrariesRepr>::new(selfref.state);
      let _ = seq.next_element_seed(cogseed)?
        .ok_or_else(|| de::Error::invalid_length(7, selfref))?;
    },{
      let cogseed = CognitionDeserializeSeed::<ForeignLibrariesRepr>::new(selfref.state);
      let _ = map.next_value_seed(cogseed)?;
      fllibs = true;
    })
  }
}

pub fn deserialize_cognition_state<'de, D>(deserializer: D, ignore_fllibs: bool) -> Result<CognitionState, D::Error>
where D: Deserializer<'de>
{
  let mut state = cogstate_init();
  deserialize_cognition_state_from_state(deserializer, &mut state, ignore_fllibs)?;
  Ok(state)
}

pub fn deserialize_cognition_state_from_state_with_fllibs<'de, D, F>(deserializer: D, fllib_deserializer: F, state: &mut CognitionState) -> Result<(), TripleErr<'de, D, F>>
where
  D: Deserializer<'de>,
  F: Deserializer<'de>
{
  match serde_load_fllibs(fllib_deserializer, state) {
    Err(e) => return Err(TripleErr::F(e)),
    Ok(opt) => if let Some(e) = opt { return Err(TripleErr::S(e)) }
  }
  if let Err(e) = deserialize_cognition_state_from_state(deserializer, state, true) {
    return Err(TripleErr::D(e))
  }
  Ok(())
}

pub fn deserialize_cognition_state_with_fllibs<'de, D, F>(deserializer: D, fllib_deserializer: F) -> Result<CognitionState, TripleErr<'de, D, F>>
where
  D: Deserializer<'de>,
  F: Deserializer<'de>
{
  let mut state = cogstate_init();
  deserialize_cognition_state_from_state_with_fllibs(deserializer, fllib_deserializer, &mut state)?;
  Ok(state)
}
