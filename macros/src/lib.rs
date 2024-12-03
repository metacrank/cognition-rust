use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, parse_str, parse_quote, Error, ImplItem, ItemImpl, LitStr, Token, Type, TypePath};

mod kw {
  syn::custom_keyword!(name);
  syn::custom_keyword!(serde_as_void);
  syn::custom_keyword!(cognition_serde);
  syn::custom_keyword!(option_serde);
}

struct CustomArgs {
  pub name: Option<LitStr>,
  pub serde_as_void: bool,
  pub cognition_serde: bool,
  pub option_serde: bool
}

macro_rules! parse_name {
  ($input:ident,$args:ident) => {
    $input.parse::<kw::name>()?;
    $input.parse::<Token![=]>()?;
    let name: LitStr = $input.parse()?;
    $input.parse::<Option<Token![,]>>()?;
    $args.name = Some(name);
  }
}
macro_rules! parse_serde_as_void {
  ($input:ident,$args:ident) => {
    $input.parse::<kw::serde_as_void>()?;
    $input.parse::<Option<Token![,]>>()?;
    $args.serde_as_void = true;
  }
}
macro_rules! parse_cognition_serde {
  ($input:ident,$args:ident) => {
    $input.parse::<kw::cognition_serde>()?;
    $input.parse::<Option<Token![,]>>()?;
    $args.cognition_serde = true;
  }
}
macro_rules! parse_option_serde {
  ($input:ident,$args:ident) => {
    $input.parse::<kw::option_serde>()?;
    $input.parse::<Option<Token![,]>>()?;
    $args.option_serde = true;
  }
}

impl Parse for CustomArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut args = CustomArgs{ name: None, serde_as_void: false, cognition_serde: false, option_serde: false };
    if !input.is_empty() {
      if input.peek(kw::name) {
        parse_name!(input, args);
        if input.peek(kw::serde_as_void) {
          parse_serde_as_void!(input, args);
          if input.peek(kw::cognition_serde) {
            parse_cognition_serde!(input, args);
            if input.peek(kw::option_serde) {
              parse_option_serde!(input, args);
            }
          } else if input.peek(kw::option_serde) {
            parse_option_serde!(input, args);
            if input.peek(kw::cognition_serde) {
              parse_cognition_serde!(input, args);
            }
          }
        } else if input.peek(kw::cognition_serde) {
          parse_cognition_serde!(input, args);
          if input.peek(kw::serde_as_void) {
            parse_serde_as_void!(input, args);
            if input.peek(kw::option_serde) {
              parse_option_serde!(input, args);
            }
          } else if input.peek(kw::option_serde) {
            parse_option_serde!(input, args);
            if input.peek(kw::serde_as_void) {
              parse_serde_as_void!(input, args);
            }
          }
        } else if input.peek(kw::option_serde) {
          parse_option_serde!(input, args);
          if input.peek(kw::serde_as_void) {
            parse_serde_as_void!(input, args);
            if input.peek(kw::cognition_serde) {
              parse_cognition_serde!(input, args);
            }
          } else if input.peek(kw::cognition_serde) {
            parse_cognition_serde!(input, args);
            if input.peek(kw::serde_as_void) {
              parse_serde_as_void!(input, args);
            }
          }
        }
      } else if input.peek(kw::serde_as_void) {
        parse_serde_as_void!(input, args);
        if input.peek(kw::name) {
          if input.peek(kw::cognition_serde) {
            parse_cognition_serde!(input, args);
            if input.peek(kw::option_serde) {
              parse_option_serde!(input, args);
            }
          } else if input.peek(kw::option_serde) {
            parse_option_serde!(input, args);
            if input.peek(kw::cognition_serde) {
              parse_cognition_serde!(input, args);
            }
          }
        } else if input.peek(kw::cognition_serde) {
          parse_cognition_serde!(input, args);
          if input.peek(kw::name) {
            parse_name!(input, args);
            if input.peek(kw::option_serde) {
              parse_option_serde!(input, args);
            }
          } else if input.peek(kw::option_serde) {
            parse_option_serde!(input, args);
            if input.peek(kw::name) {
              parse_name!(input, args);
            }
          }
        } else if input.peek(kw::option_serde) {
          parse_option_serde!(input, args);
          if input.peek(kw::name) {
            parse_name!(input, args);
            if input.peek(kw::cognition_serde) {
              parse_cognition_serde!(input, args);
            }
          } else if input.peek(kw::cognition_serde) {
            parse_cognition_serde!(input, args);
            if input.peek(kw::name) {
              parse_name!(input, args);
            }
          }
        }
      } else if input.peek(kw::cognition_serde) {
        parse_cognition_serde!(input, args);
        if input.peek(kw::name) {
          parse_name!(input, args);
          if input.peek(kw::serde_as_void) {
            parse_serde_as_void!(input, args);
            if input.peek(kw::option_serde) {
              parse_option_serde!(input, args);
            }
          } else if input.peek(kw::option_serde) {
            parse_option_serde!(input, args);
            if input.peek(kw::serde_as_void) {
              parse_serde_as_void!(input, args);
            }
          }
        } else if input.peek(kw::serde_as_void) {
          parse_serde_as_void!(input, args);
          if input.peek(kw::name) {
            parse_name!(input, args);
            if input.peek(kw::option_serde) {
              parse_option_serde!(input, args);
            }
          } else if input.peek(kw::option_serde) {
            parse_option_serde!(input, args);
            if input.peek(kw::name) {
              parse_name!(input, args);
            }
          }
        } else if input.peek(kw::option_serde) {
          parse_option_serde!(input, args);
          if input.peek(kw::name) {
            parse_name!(input, args);
            if input.peek(kw::serde_as_void) {
              parse_serde_as_void!(input, args);
            }
          } else if input.peek(kw::serde_as_void) {
            parse_serde_as_void!(input, args);
            if input.peek(kw::name) {
              parse_name!(input, args);
            }
          }
        }
      } else if input.peek(kw::option_serde) {
        parse_option_serde!(input, args);
        if input.peek(kw::name) {
          parse_name!(input, args);
          if input.peek(kw::serde_as_void) {
            parse_serde_as_void!(input, args);
            if input.peek(kw::cognition_serde) {
              parse_cognition_serde!(input, args);
            }
          } else if input.peek(kw::cognition_serde) {
            parse_cognition_serde!(input, args);
            if input.peek(kw::serde_as_void) {
              parse_serde_as_void!(input, args);
            }
          }
        } else if input.peek(kw::serde_as_void) {
          parse_serde_as_void!(input, args);
          if input.peek(kw::name) {
            parse_name!(input, args);
            if input.peek(kw::cognition_serde) {
              parse_cognition_serde!(input, args);
            }
          } else if input.peek(kw::cognition_serde) {
            parse_cognition_serde!(input, args);
            if input.peek(kw::name) {
              parse_name!(input, args);
            }
          }
        } else if input.peek(kw::cognition_serde) {
          parse_cognition_serde!(input, args);
          if input.peek(kw::name) {
            parse_name!(input, args);
            if input.peek(kw::serde_as_void) {
              parse_serde_as_void!(input, args);
            }
          } else if input.peek(kw::serde_as_void) {
            parse_serde_as_void!(input, args);
            if input.peek(kw::name) {
              parse_name!(input, args);
            }
          }
        }
      } else {
        return Err(input.error("invalid argument to custom proc macro"))
      }
    };
    Ok(args)
  }
}

#[proc_macro_attribute]
pub fn custom(args: TokenStream, input: TokenStream) -> TokenStream {
  let args: CustomArgs = parse_macro_input!(args as CustomArgs);
  let mut input = parse_macro_input!(input as ItemImpl);

  let (name, custom_type, snake_name) = match &args.name {
    Some(n) => (quote!{ #n }, parse_str::<Type>(n.suffix()).unwrap(), snake_name(n.suffix())),
    None => match type_name(&input.self_ty) {
      Some(n) => (quote!{ #n }, parse_str::<Type>(&n).unwrap(), snake_name(&n)),
      None => {
        let msg = "Custom impl must specify a custom type with syn::Type::Path or syn::Type::Group";
        return Error::new_spanned(&input.self_ty, msg).to_compile_error().into();
      }
    }
  };

  let serde_as_void = args.serde_as_void;

  input.items.push(parse_quote! { fn as_any(&self) -> &dyn Any { self } });
  input.items.push(parse_quote! { fn as_any_mut(&mut self) -> &mut dyn Any { self } });
  input.items.push(parse_quote! {
    fn custom_type_name(&self) -> &'static str {
      concat!(module_path!(), "::", #name)
    }
  });

  add_custom_pool(&mut input);

  let mut expanded = if args.serde_as_void {
    void_serde(name, custom_type.clone())
  } else if args.cognition_serde {
    cognition_serde(name, custom_type.clone())
  } else if args.option_serde {
    option_serde(name, custom_type.clone())
  } else {
    default_serde(name, custom_type.clone())
  };

  if serde_as_void { add_serialize(&mut expanded, custom_type.clone()) }
  expanded.extend(quote!{ #input });

  add_downcast(&mut expanded, &snake_name, custom_type);

  expanded.into()
}

fn add_custom_pool(input: &mut ItemImpl) {
  let cond = |item: &ImplItem| 'ret: {
    if let ImplItem::Fn(f) = item {
      if f.sig.ident == Ident::new("custom_pool", Span::call_site()) {
        break 'ret false
      }
    }
    true
  };

  if input.items.iter().all(cond) {
    input.items.push(parse_quote! {
      fn custom_pool(&mut self, _: &mut Pool) -> CustomPoolPackage { CustomPoolPackage::None }
    })
  }
}

fn void_serde(name: proc_macro2::TokenStream, custom_type: Type) -> proc_macro2::TokenStream {
  quote! {
    impl CustomTypeData for #custom_type {
      fn custom_type_name() -> &'static str {
        concat!(module_path!(), "::", #name)
      }
      fn deserialize_fn() -> DeserializeFn<dyn Custom> {
        (|deserializer, _state| Ok(
          Box::new(erased_serde::deserialize::<Void>(deserializer)?),
        )) as DeserializeFn<dyn Custom>
      }
    }
  }
}

fn cognition_serde(name: proc_macro2::TokenStream, custom_type: Type) -> proc_macro2::TokenStream {
  quote! {
    impl CustomTypeData for #custom_type {
      fn custom_type_name() -> &'static str {
        concat!(module_path!(), "::", #name)
      }
      fn deserialize_fn() -> DeserializeFn<dyn Custom> {
        (|deserializer, state| Ok(
          Box::new(<#custom_type as CognitionDeserialize>::cognition_deserialize(deserializer, state)?),
        )) as DeserializeFn<dyn Custom>
      }
    }
  }
}

fn option_serde(name: proc_macro2::TokenStream, custom_type: Type) -> proc_macro2::TokenStream {
  quote! {
    impl CustomTypeData for #custom_type {
      fn custom_type_name() -> &'static str {
        concat!(module_path!(), "::", #name)
      }
      fn deserialize_fn() -> DeserializeFn<dyn Custom> {
        (|deserializer, state| Ok(
          match <#custom_type as OptionDeserialize>::option_deserialize(deserializer, state)? {
            Some(c) => Box::new(c),
            None => Box::new(Void{})
          }
        )) as DeserializeFn<dyn Custom>
      }
    }
  }
}

fn default_serde(name: proc_macro2::TokenStream, custom_type: Type) -> proc_macro2::TokenStream {
  quote! {
    impl CustomTypeData for #custom_type {
      fn custom_type_name() -> &'static str {
        concat!(module_path!(), "::", #name)
      }
      fn deserialize_fn() -> DeserializeFn<dyn Custom> {
        (|deserializer, _state| Ok(
          Box::new(erased_serde::deserialize::<#custom_type>(deserializer)?),
        )) as DeserializeFn<dyn Custom>
      }
    }
  }
}

fn add_serialize(expanded: &mut proc_macro2::TokenStream, custom_type: Type) {
  expanded.extend(quote! {
    impl ::serde::ser::Serialize for #custom_type {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: ::serde::ser::Serializer,
      { Void::serialize(&Void{}, serializer) }
    }
  });
}

fn add_downcast(expanded: &mut proc_macro2::TokenStream, snake_name: &str, custom_type: Type) {
  let downcast_ref = downcast_snake(snake_name, "_ref");
  let downcast_mut = downcast_snake(snake_name, "_mut");

  expanded.extend(quote! {
    pub fn #downcast_ref(custom: &dyn Custom) -> Option<&#custom_type> {
      custom.as_any().downcast_ref::<#custom_type>()
    }
    pub fn #downcast_mut(custom: &mut dyn Custom) -> Option<&mut #custom_type> {
      custom.as_any_mut().downcast_mut::<#custom_type>()
    }
  });
}

fn downcast_snake(snake_name: &str, ext: &str) -> proc_macro2::TokenStream {
  let mut downcast = String::from("downcast_");
  downcast.push_str(snake_name);
  downcast.push_str(ext);
  let identifier = Ident::new(&downcast, Span::call_site());
  quote!{ #identifier }
}

fn snake_name(name: &str) -> String {
  let mut snake_name = String::new();
  let mut iter = name.chars();
  if let Some(c) = iter.next() {
    snake_name.push(c.to_ascii_lowercase())
  }
  for c in iter {
    if c.is_ascii_uppercase() {
      snake_name.push('_');
      snake_name.push(c.to_ascii_lowercase())
    } else {
      snake_name.push(c.clone());
    }
  }
  snake_name
}

fn type_name(mut ty: &Type) -> Option<String> {
  loop {
    match ty {
      Type::Path(TypePath{ qself: None, path }) => {
        break Some(path.segments.last().unwrap().ident.to_string())
      },
      Type::Group(group) => ty = &group.elem,
      _ => break None
    }
  }
}
