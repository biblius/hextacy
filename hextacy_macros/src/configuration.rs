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

    let field_len = strct.fields.len();

    for field in strct.fields {
        let field_info = FieldInfo::new(&field, &input.ident);
        let field_id = field_info.id.clone();
        let mut field_loader = FieldLoader::new(field_info);

        // Parse attributes
        let mut priority = 0;
        for attr in field.attrs {
            // Parse env
            if attr.meta.path().is_ident("env") {
                let mut env_loader = EnvLoader::from(&attr.meta);
                env_loader.priority = priority;
                let loader = Box::new(env_loader);
                field_loader
                    .loaders
                    .entry(field_id.clone())
                    .or_insert(vec![])
                    .push(loader);
                priority += 1;
            }

            // Parse raw
            if attr.meta.path().is_ident("raw") {
                let mut raw_loader = RawLoader::from(&attr.meta);
                raw_loader.priority = priority;
                let loader = Box::new(raw_loader);
                field_loader
                    .loaders
                    .entry(field_id.clone())
                    .or_insert(vec![])
                    .push(loader);
                priority += 1;
            }

            // Parse helpers

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

    // Sort each field loader collection by priority
    for f_loader in field_loaders.iter_mut() {
        let v = &f_loader.field.id;
        f_loader
            .loaders
            .get_mut(v)
            .unwrap()
            .sort_by_key(|a| a.priority())
    }

    let mut tokens = TokenStream::new();

    tokens.append_all(&field_loaders);

    let config_struct = &input.ident;
    let (imp, ty, wher) = input.generics.split_for_impl();
    let asyncness = field_loaders
        .iter()
        .any(|l| l.is_async)
        .then_some(quote!(async));

    let loader_calls = field_loaders
        .iter()
        .map(|l| {
            let var = &l.field.id;
            let loaders = l.loaders.get(var).unwrap();

            let mut last_err = quote!();
            let len = loaders.len();

            if l.is_async {
                let mut quoted = quote!();
                for (i, loader) in loaders.iter().enumerate() {
                    let loader_fn = loader.fn_ident(var);
                    let log_err = loader.error_log();
                    if i == len - 1 {
                        quoted.extend(quote!(
                            let #var = #loader_fn().await?;
                        ));
                        break;
                    }
                    quoted.extend(quote!(
                        let #var = #loader_fn().await;
                        if let Err(e) = #var {
                            #log_err;
                        }
                    ));
                }

                return quoted;
            }

            let mut quoted = quote!( let #var = );
            for (i, loader) in loaders.iter().enumerate() {
                let loader_fn = loader.fn_ident(var);

                if i == 0 {
                    last_err = loader.error_log();
                    quoted.extend(quote!( #loader_fn() ));
                    if len == 1 {
                        quoted.extend(quote!(?;));
                    }
                    continue;
                }

                quoted.extend(quote!(
                    .or_else(|e| {
                        #last_err;
                        #loader_fn()
                    })
                ));

                last_err = loader.error_log();

                if i == len - 1 {
                    quoted.extend(quote!(?;));
                }
            }

            quoted
        })
        .collect::<Vec<_>>();

    let self_fields = field_loaders
        .iter()
        .map(|el| el.field.id.clone())
        .collect::<Vec<_>>();

    let error_id = format_ident!("{}ConfigurationError", config_struct);

    let error = quote!(
        /// Autogenerated with `#[derive(Configuration)]`
        #[derive(Debug)]
        pub enum #error_id {
            Env(String),
            Raw
        }

        impl std::fmt::Display for #error_id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    AppStateConfigurationError::Env(s) => write!(f, "{s}"),
                    AppStateConfigurationError::Raw => unreachable!(),
                }
            }
        }
    );

    let configure_fn = quote!(
        impl #imp #config_struct #ty #wher {
            /// Initialises the struct by calling all functions generated by `Configure`.
            pub #asyncness fn configure() -> Result<Self, #error_id> {
                #(#loader_calls ; )*
                Ok(Self {
                    #(#self_fields),*
                })
            }
        }
    );

    tokens.extend(error);

    if field_len == field_loaders.len() {
        tokens.extend(configure_fn);
    }

    Ok(tokens)
}

trait Loader: std::fmt::Debug {
    fn fn_ident(&self, field_id: &Ident) -> Ident;

    fn priority(&self) -> usize;

    fn extend_tokens(
        &self,
        field: &FieldInfo,
        is_async: bool,
        load_with: Option<&syn::Path>,
        tokens: &mut TokenStream,
    );

    fn error_variant(&self, config_id: &Ident) -> TokenStream;

    fn error_log(&self) -> TokenStream;
}

/// Top level loader that collects config loaders on a per field basis.
#[derive(Debug)]
struct FieldLoader {
    field: FieldInfo,
    loaders: HashMap<Ident, Vec<Box<dyn Loader>>>,
    is_async: bool,
    load_with: Option<syn::Path>,
}

impl FieldLoader {
    fn new(field: FieldInfo) -> Self {
        Self {
            field,
            loaders: HashMap::new(),
            is_async: false,
            load_with: None,
        }
    }
}

#[derive(Debug)]
struct FieldInfo {
    id: Ident,
    strct: PathSegment,
    wrappers: Vec<Ident>,
    config_struct: Ident,
}

impl FieldInfo {
    fn new(field: &Field, config_struct: &Ident) -> Self {
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
            config_struct: config_struct.clone(),
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
    priority: usize,
    keys: Vec<EnvVar>,
}

impl From<&Meta> for EnvLoader {
    fn from(meta: &Meta) -> Self {
        let mut loader = Self {
            keys: vec![],
            priority: 0,
        };

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
    fn fn_ident(&self, field_id: &Ident) -> Ident {
        format_ident!("init_{field_id}_env")
    }

    fn priority(&self) -> usize {
        self.priority
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
            config_struct,
        } = field;
        let id = self.fn_ident(id);
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

                let err_variant = self.error_variant(config_struct);

                let required_msg = format!("Required variable {env_var} not found in env");
                let required_err = quote!(#err_variant(#required_msg.to_string()));

                let parse_msg = format!("Could not parse {env_var} to specified type");
                let parse_err = quote!(#err_variant(#parse_msg.to_string()));

                let convert = match (env_key.optional, env_key.parse_to.as_ref()) {
                    (true, Some(to)) => quote!(
                        // This ensures an error is returned on unparseable values
                        ; if let Some(val) = #id {
                             if val.parse::<#to>().is_err() {
                                 return Err(#parse_err)
                                }
                            }
                        let #id = #id.map(|x|x.parse::<#to>().unwrap());
                    ),
                    (true, None) => quote!(.map(|x| x.as_str());),
                    (false, Some(to)) => {
                        quote!(.ok_or(#required_err)?.parse::<#to>().map_err(|_|#parse_err)?;)
                    }
                    (false, None) => quote!(.map(|x| x.as_str()).ok_or(#required_err)?;),
                };

                quote!(let #id = params.get(#env_var) #convert)
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

        let config_err = format_ident!("{config_struct}ConfigurationError");

        let quoted = quote!(
            #async_fn fn #id () -> Result<#return_ty, #config_err> {
                let params = ::hextacy::config::env::get_multiple(&[#( #env_keys ),*]);
                #(#variables)*
                Ok(#constructor)
            }
        );

        tokens.extend(quoted)
    }

    fn error_variant(&self, config_id: &Ident) -> TokenStream {
        let err = format_ident!("{config_id}ConfigurationError");
        quote!(#err::Env)
    }

    fn error_log(&self) -> TokenStream {
        quote!(hextacy::error!(
            "Error occurred while loading from env: {e}"
        ))
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
    priority: usize,
    values: Vec<Expr>,
}

impl From<&Meta> for RawLoader {
    fn from(meta: &Meta) -> Self {
        let mut loader = Self {
            values: vec![],
            priority: 0,
        };

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
    fn fn_ident(&self, field_id: &Ident) -> Ident {
        format_ident!("init_{field_id}_raw")
    }

    fn priority(&self) -> usize {
        self.priority
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
            config_struct,
        } = field;
        let id = self.fn_ident(id);
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

        let config_err = format_ident!("{config_struct}ConfigurationError");

        let quoted = quote!(
            /// This function will never error
            #async_fn fn #id () -> Result<#return_ty, #config_err> {
                Ok(#constructor)
            }
        );

        tokens.extend(quoted)
    }

    // Raw loaders can never error since an invalid configuration will be stopped at compile time
    fn error_variant(&self, config_id: &Ident) -> TokenStream {
        let err = format_ident!("{config_id}ConfigurationError");
        quote!(#err::Raw)
    }

    fn error_log(&self) -> TokenStream {
        quote!(hextacy::error!("Error occurred while loading raw: {e}"))
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
