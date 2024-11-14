use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, parse_str, parse_quote, Error, ItemImpl, LitStr, Token, Type, TypePath};

mod kw {
  syn::custom_keyword!(name);
  syn::custom_keyword!(serde_as_void);
}

struct CustomArgs {
  pub name: Option<LitStr>,
  pub serde_as_void: bool,
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

impl Parse for CustomArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut args = CustomArgs{ name: None, serde_as_void: false };
    if !input.is_empty() {
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
  //let args = CustomArgs{ name: None, serde_as_void: false };
  let mut input = parse_macro_input!(input as ItemImpl);

  let (name, custom_type) = match &args.name {
    Some(n) => (quote!{ #n }, parse_str::<Type>(n.suffix()).unwrap()),
    None => match type_name(&input.self_ty) {
      Some(n) => (quote!{ #n }, parse_str::<Type>(&n).unwrap()),
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

  let mut expanded = if args.serde_as_void {
    quote! {
      impl CustomTypeData for #custom_type {
        fn custom_type_name() -> &'static str {
          concat!(module_path!(), "::", #name)
        }
        fn deserialize_fn() -> DeserializeFn<dyn Custom> {
          (|deserializer| Ok(
            Box::new(erased_serde::deserialize::<Void>(deserializer)?),
          )) as DeserializeFn<dyn Custom>
        }
      }
    }
  } else {
    quote! {
      impl CustomTypeData for #custom_type {
        fn custom_type_name() -> &'static str {
          concat!(module_path!(), "::", #name)
        }
        fn deserialize_fn() -> DeserializeFn<dyn Custom> {
          (|deserializer| Ok(
            Box::new(erased_serde::deserialize::<#custom_type>(deserializer)?),
          )) as DeserializeFn<dyn Custom>
        }
      }
    }
  };

  if serde_as_void {
    expanded.extend(quote! {
      impl ::serde::ser::Serialize for #custom_type {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
          S: ::serde::ser::Serializer,
        { Void::serialize(&Void{}, serializer) }
      }
    });
  }
  expanded.extend(quote!{ #input });
  expanded.into()
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
