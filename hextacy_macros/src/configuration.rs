use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, DeriveInput, Expr, Field, Ident,
    LitStr, Meta, PathArguments, PathSegment, Token, Type,
};

pub fn impl_configuration(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Data::Struct(strct) = input.data else {
        abort!(input.span(), "Configuration derive only works for structs");
    };

    let mut field_loaders = vec![];

    for field in strct.fields {
        let field_info = FieldInfo::from(&field);
        let field_id = field_info.id.clone();

        let mut field_loader = FieldLoader {
            field: field_info,
            loaders: HashMap::new(),
            is_async: false,
            load_with: None,
        };

        // Parse attributes
        for attr in field.attrs {
            // Parse env
            if attr.meta.path().is_ident("env") {
                let env_loader = EnvLoader::from(&attr.meta);
                let loader = Box::new(env_loader);
                field_loader
                    .loaders
                    .entry(field_id.clone())
                    .or_insert(vec![])
                    .push(loader);
            }

            // Parse raw
            if attr.meta.path().is_ident("raw") {
                let raw_loader = RawLoader::from(&attr.meta);
                let loader = Box::new(raw_loader);
                field_loader
                    .loaders
                    .entry(field_id.clone())
                    .or_insert(vec![])
                    .push(loader);
            }

            if attr.meta.path().is_ident("load_async") {
                field_loader.is_async = true;
            }

            if attr.meta.path().is_ident("load_with") {
                let list = attr.meta.require_list()?;
                field_loader.load_with = Some(list.parse_args::<syn::Path>()?);
            }
        }

        field_loaders.push(field_loader);
    }

    field_loaders.retain(|loader| !loader.loaders.is_empty());

    let mut tokens = TokenStream::new();

    tokens.append_all(field_loaders);

    Ok(tokens)
}

trait Loader: std::fmt::Debug {
    fn fn_ident(&self, field_id: Ident) -> Ident;

    fn extend_tokens(
        &self,
        field: &FieldInfo,
        is_async: bool,
        load_with: Option<&syn::Path>,
        tokens: &mut TokenStream,
    );
}

/// Top level loader that collects config loaders on a per field basis.
#[derive(Debug)]
struct FieldLoader {
    field: FieldInfo,
    loaders: HashMap<Ident, Vec<Box<dyn Loader>>>,
    is_async: bool,
    load_with: Option<syn::Path>,
}

#[derive(Debug)]
struct FieldInfo {
    id: Ident,
    strct: PathSegment,
    wrappers: Vec<Ident>,
}

impl From<&Field> for FieldInfo {
    fn from(field: &Field) -> Self {
        let field_id = field.ident.as_ref().unwrap_or_else(|| {
            abort!(
                field.span(),
                "Configuration macro must be used on named structs"
            )
        });

        let mut wrappers = vec![];

        let Type::Path(ref p) = field.ty else {
            abort!(
                field.ty.span(),
                "`env` loader cannot be implemented for type"
            )
        };

        let seg = p
            .path
            .segments
            .last()
            .unwrap_or_else(|| abort!(p.path.segments.span(), "Wrapper not supported"));

        let original = find_original(seg, &mut wrappers);

        Self {
            id: field_id.clone(),
            strct: original,
            wrappers,
        }
    }
}

impl ToTokens for FieldLoader {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            field,
            loaders,
            is_async,
            load_with,
        } = &self;

        let loaders = loaders.get(&field.id).unwrap();
        for loader in loaders {
            loader.extend_tokens(field, *is_async, load_with.as_ref(), tokens)
        }
    }
}

/// Loading strategy for the `env` attribute.
#[derive(Debug)]
struct EnvLoader {
    keys: Vec<EnvVar>,
}

impl From<&Meta> for EnvLoader {
    fn from(meta: &Meta) -> Self {
        let mut loader = Self { keys: vec![] };

        let list = meta.require_list().unwrap_or_else(|_| {
            abort!(
                meta.span(),
                "`env` loader must be a list of variables to load from the env"
            )
        });

        loader.keys.extend(
            list.parse_args_with(Punctuated::<EnvVar, Token![,]>::parse_terminated)
                .unwrap()
                .into_pairs()
                .map(|pair| pair.into_value()),
        );

        loader
    }
}

impl Loader for EnvLoader {
    fn fn_ident(&self, field_id: Ident) -> Ident {
        format_ident!("init_{field_id}_env")
    }

    fn extend_tokens(
        &self,
        field: &FieldInfo,
        is_async: bool,
        load_with: Option<&syn::Path>,
        tokens: &mut TokenStream,
    ) {
        let FieldInfo {
            id,
            strct,
            wrappers,
        } = field;
        let id = self.fn_ident(id.clone());
        let env_keys = &self.keys;

        let to_var_ident = |var: &EnvVar| Ident::new(&var.lit.to_lowercase(), Span::call_site());

        // Account for asyncness
        let (async_fn, async_constr) = if is_async {
            (quote!(async), quote!(.await))
        } else {
            (quote!(), quote!())
        };

        // Custom constructor
        let constructor_fn = load_with.map(|p| quote!(#p)).unwrap_or(quote!(#strct::new));

        // For the call to strct::new
        let constructor_vars = env_keys.iter().map(to_var_ident).collect::<Vec<_>>();

        // The variable names and their conversion
        let variables = env_keys
            .iter()
            .map(|env_key| {
                let id = to_var_ident(env_key);
                let env_var = &env_key.lit;

                let convert_err = format!("Required variable {} not found in env, if it should be optional, denote it with `\"{}\" as Option`", env_key.lit, env_key.lit);
                let parse_err = format!("Could not parse \"{}\" to specified type", env_key.lit);

                // Handles required fields
                let convert = match (env_key.optional, env_key.parse_to.is_some()) {
                    (true, true) => quote!(),
                    (true, false) => quote!(.map(|x| x.as_str())),
                    (false, true) => quote!(.expect(#convert_err)),
                    (false, false) => quote!(.map(|x| x.as_str()).expect(#convert_err)),
                };

                let parse = env_key
                    .parse_to
                    .as_ref()
                    .map(|to| quote!( .map(|var| var.parse::<#to>().expect(#parse_err)) ))
                    .unwrap_or(quote!());

                quote!(let #id = params.get(#env_var) #parse #convert ;)
            })
            .collect::<Vec<_>>();

        // For collecting the vars with get_multiple
        let env_keys = env_keys.iter().map(|k| k.lit.clone()).collect::<Vec<_>>();

        let mut return_ty = quote!(#strct);
        for wrapper in wrappers {
            return_ty = quote!(#wrapper<#return_ty>);
        }

        let mut constructor = quote!( #constructor_fn ( #( #constructor_vars ),* ) #async_constr);
        for wrapper in wrappers {
            constructor = quote!(#wrapper::new(#constructor));
        }

        let quoted = quote!(
            #async_fn fn #id () -> #return_ty {
                let params = ::hextacy::config::env::get_multiple(&[#( #env_keys ),*]);
                #(#variables)*
                #constructor
            }
        );

        tokens.extend(quoted)
    }
}

/// Represent a variable key in the `env` attribute.
#[derive(Debug)]
struct EnvVar {
    lit: String,
    parse_to: Option<syn::Type>,
    optional: bool,
}

impl Parse for EnvVar {
    /// Intended to be used in conjuction with Punctuated so we don't
    /// take into account the commas.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Self {
            lit: String::new(),
            parse_to: None,
            optional: false,
        };

        this.lit = input.parse::<LitStr>()?.value();

        if input.is_empty() {
            return Ok(this);
        }

        let lit_next = input.peek2(LitStr);

        if lit_next {
            return Ok(this);
        }

        input.parse::<Token!(as)>()?;
        let ty = input.parse::<syn::Type>()?;
        let syn::Type::Path(ref path) = ty else {
            return Err(syn::Error::new(
                ty.span(),
                "Expected either type that can be parsed from String or `Option`",
            ));
        };

        let segment = path.path.segments.first().unwrap();

        if segment.ident == "Option" {
            // Found option, check for conversion
            this.optional = true;

            let args = match &segment.arguments {
                PathArguments::AngleBracketed(ab) => ab,
                // No args means no parsing
                PathArguments::None => return Ok(this),
                PathArguments::Parenthesized(_) => {
                    return Err(syn::Error::new(
                        segment.arguments.span(),
                        "Expected Option<T>",
                    ))
                }
            };

            let Some(ty) = args.args.first() else {
                return Err(syn::Error::new(
                    args.args.span(),
                    format!("Must specify type for parsing, if you do not need to parse, use `\"{}\" as Option`", this.lit),
                ));
            };

            let syn::GenericArgument::Type(ty) = ty else {
                return Err(syn::Error::new(segment.arguments.span(), "Expected type"));
            };

            this.parse_to = Some(ty.clone());
        } else {
            this.parse_to = Some(ty);
        }

        Ok(this)
    }
}

/// Loading strategy for the `raw` attribute. Parses all valid `Expr`s.
#[derive(Debug)]
struct RawLoader {
    values: Vec<Expr>,
}

impl From<&Meta> for RawLoader {
    fn from(meta: &Meta) -> Self {
        let mut loader = Self { values: vec![] };

        let list = meta.require_list().unwrap_or_else(|_| {
            abort!(
                meta.span(),
                "`env` loader must be a list of variables to load from the env"
            )
        });

        loader.values.extend(
            list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                .unwrap()
                .into_pairs()
                .map(|pair| pair.into_value()),
        );

        loader
    }
}

impl Loader for RawLoader {
    fn fn_ident(&self, field_id: Ident) -> Ident {
        format_ident!("init_{field_id}_raw")
    }

    fn extend_tokens(
        &self,
        field: &FieldInfo,
        is_async: bool,
        load_with: Option<&syn::Path>,
        tokens: &mut TokenStream,
    ) {
        let FieldInfo {
            id,
            strct,
            wrappers,
        } = field;
        let id = self.fn_ident(id.clone());
        let args = &self.values;

        let (async_fn, async_constr) = if is_async {
            (quote!(async), quote!(.await))
        } else {
            (quote!(), quote!())
        };

        let constructor_fn = load_with.map(|p| quote!(#p)).unwrap_or(quote!(#strct::new));

        let mut return_ty = quote!(#strct);
        for wrapper in wrappers {
            return_ty = quote!(#wrapper<#return_ty>);
        }

        let mut constructor = quote!(#constructor_fn ( #( #args ),* ) #async_constr);
        for wrapper in wrappers {
            constructor = quote!(#wrapper::new(#constructor));
        }

        let quoted = quote!(
            #async_fn fn #id () -> #return_ty {
                #constructor
            }
        );

        tokens.extend(quoted)
    }
}

/// Recursively goes through wrappers until it finds one with no AB args and returns it, this will usually be the
/// struct in question.
fn find_original(seg: &PathSegment, wrappers: &mut Vec<Ident>) -> PathSegment {
    match seg.arguments {
        PathArguments::None => seg.clone(),
        PathArguments::AngleBracketed(ref ab) => {
            wrappers.insert(0, seg.ident.clone());
            let arg = ab
                .args
                .last()
                .unwrap_or_else(|| abort!(ab.args.span(), "Wrapper not supported"));
            match arg {
                syn::GenericArgument::Type(Type::Path(p)) => {
                    let s = p.path.segments.last().unwrap_or_else(|| {
                        abort!(
                            p.path.segments.span(),
                            "`env` loader not supported for type"
                        )
                    });
                    find_original(s, wrappers)
                }
                _ => abort!(arg.span(), "`env` loader not supported for type"),
            }
        }
        PathArguments::Parenthesized(ref p) => {
            abort!(p.span(), "`env` loader not supported for type")
        }
    }
}
