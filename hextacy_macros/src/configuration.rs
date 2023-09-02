use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, DeriveInput, Ident, LitStr,
    PathArguments, PathSegment, Token, Type, 
};

pub fn impl_configuration(input: DeriveInput) -> proc_macro2::TokenStream {
    let syn::Data::Struct(strct) = input.data else {
        unimplemented!("Configuration derive only works for structs");
    };

    let mut loaders = vec![];

    for field in strct.fields {
        let field_id = field.ident.clone().unwrap_or_else(|| {
            abort!(
                field.ident.span(),
                "Configuration macro must be used on named structs"
            )
        });

        let mut wrappers = vec![];

        let Type::Path(p) = field.ty.clone() else {
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

        let original = find_original(seg.clone(), &mut wrappers);

        let mut field_loader = FieldLoader {
            id: field_id,
            strct: original,
            wrappers,
            env_keys: vec![],
            source: None,
        };

        // Parse attributes
        for attr in field.attrs {
            if attr.meta.path().is_ident("env") {
                field_loader.source = Some(VarSource::Env);
                let list = attr.meta.require_list().unwrap_or_else(|_| {
                    abort!(
                        attr.meta.span(),
                        "`env` loader must be a list of variables to load from the env"
                    )
                });

                field_loader.env_keys.extend(
                    list.parse_args_with(Punctuated::<EnvVar, Token![,]>::parse_terminated)
                        .unwrap()
                        .into_pairs()
                        .map(|pair| pair.into_value()),
                );
            }
        }

        loaders.push(field_loader);
    }

    loaders.retain(|loader| !loader.env_keys.is_empty());

    let mut tokens = TokenStream::new();

    tokens.append_all(loaders);

    tokens
}


#[derive(Debug)]
struct FieldLoader {
    id: Ident,
    strct: PathSegment,
    wrappers: Vec<Ident>,
    env_keys: Vec<EnvVar>,
    source: Option<VarSource>,
}

impl ToTokens for FieldLoader {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            id,
            strct,
            wrappers,
            env_keys,
            source,
        } = &self;

        let source = match source.as_ref().unwrap() {
            VarSource::Env => "env",
            VarSource::Raw => "raw",
        };

        let id = format_ident!("init_{id}_{source}");

        let to_var_ident = |var: &EnvVar| Ident::new(&var.lit.to_lowercase(), Span::call_site());

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

        // For indexing to the map and fetching the vars with get_multiple
        let env_keys = env_keys
            .iter()
            .map(|k| k.lit.clone())
            .collect::<Vec<_>>();

        let mut return_ty = quote!(#strct);
        for wrapper in wrappers {
            return_ty = quote!(#wrapper<#return_ty>);
        }

        let mut constructor = quote!(#strct::new( #( #constructor_vars ),* ));
        for wrapper in wrappers {
            constructor = quote!(#wrapper::new(#constructor));
        }

        let quoted = quote!(
            fn #id () -> #return_ty {
                let params = ::hextacy::env::get_multiple(&[#( #env_keys ),*]);
                #(#variables)*
                #constructor
            }
        );

        tokens.extend(quoted)
    }
}


/// The source to which to obtain the env variables from.
#[non_exhaustive]
#[derive(Debug)]
enum VarSource {
    Env,

    // TODO
    Raw,
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

/// Recursively goes through wrappers until it finds one with no AB args and returns it, this will usually be the
/// struct in question.
fn find_original(seg: PathSegment, wrappers: &mut Vec<Ident>) -> PathSegment {
    match seg.arguments {
        PathArguments::None => seg,
        PathArguments::AngleBracketed(ab) => {
            wrappers.insert(0, seg.ident);
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
                    find_original(s.clone(), wrappers)
                }
                _ => abort!(arg.span(), "`env` loader not supported for type"),
            }
        }
        PathArguments::Parenthesized(p) => {
            abort!(p.span(), "`env` loader not supported for type")
        }
    }
}
