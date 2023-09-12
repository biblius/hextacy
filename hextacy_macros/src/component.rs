use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, Ident, ItemImpl, ItemStruct, Token, TypePath};

pub fn impl_component(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(item_impl) = syn::parse::<ItemImpl>(input.clone()) {
        let component: ComponentImpl = syn::parse(attr).expect("Could not parse component impl");
        return impl_impl_block(component, item_impl);
    }

    if let Ok(strct) = syn::parse(input.clone()) {
        let component: ComponentStruct =
            syn::parse(attr).expect("Could not parse component struct");
        return impl_struct(component, strct);
    }

    panic!("component not supported for item")
}

fn impl_impl_block(component: ComponentImpl, item_impl: ItemImpl) -> proc_macro::TokenStream {
    let attrs = item_impl.attrs;

    let existing_impl = &item_impl.generics.params;
    let existing_ty = item_impl.generics.params.iter().map(|p| match p {
        syn::GenericParam::Type(t) => t.ident.clone(),
        _ => abort!(
            item_impl.generics.params.span(),
            "component not supported for this type of impl"
        ),
    });

    let original_struct = match item_impl.self_ty.as_ref() {
        syn::Type::Path(TypePath { ref path, .. }) => &path.segments[0].ident,
        _ => abort!(
            item_impl.self_ty.span(),
            "component not supported for this type of impl"
        ),
    };

    let impl_generics = component.impl_generics();
    let ty_generics = component.ty_generics();

    let mut where_clause = quote!(where);

    component.extend_where_clause(&mut where_clause);

    if let Some(ref wher) = item_impl.generics.where_clause {
        let wher = &wher.predicates;
        where_clause.extend(quote!( #wher ))
    }

    let original_fns = &item_impl.items;

    quote!(
        #(#attrs)*
        impl<#(#impl_generics),*, #existing_impl> #original_struct <#(#ty_generics),*, #(#existing_ty),*> #where_clause
        {
            #(#original_fns)*
        }
    )
    .into()
}

fn impl_struct(component: ComponentStruct, item_struct: ItemStruct) -> proc_macro::TokenStream {
    let new_struct = quote_struct(&component, &item_struct);

    let id = &item_struct.ident;

    let existing_generics = &item_struct.generics.params;

    let generics = &component.generics();

    let mut where_clause = quote!(where);

    if let Some(ref wher) = item_struct.generics.where_clause {
        let wher = &wher.predicates;
        where_clause.extend(quote!( #wher ))
    }

    let existing_fields = item_struct.fields.iter().collect::<Vec<_>>();

    // Disgusting stuff for the new implementation because we are being
    // ass blasted by commas
    let existing_fields_for_new = existing_fields
        .iter()
        .map(|f| {
            f.ident
                .clone()
                .expect("service must be derived on structs with named fields")
        })
        .collect::<Vec<_>>();

    let existing_tys_for_new = existing_fields
        .iter()
        .map(|f| f.ty.clone())
        .collect::<Vec<_>>();

    let existing_args = (!existing_fields.is_empty()).then(|| {
        quote!(
            #( #existing_fields_for_new : #existing_tys_for_new ),*
        )
    });

    let existing_struct_fields = (!existing_fields.is_empty()).then(|| {
        quote!(
            #( #existing_fields_for_new ),*,
        )
    });

    let args_new = component.driver_and_contract_fields();
    let struct_fields_new = component.driver_contract_fields_new();

    let new = quote!(
        impl
        <#( #generics ),*, #existing_generics>
        #id
        <#( #generics ),*, #existing_generics>
        #where_clause
        {
                pub fn new(#args_new #existing_args ) -> Self {
                    Self {
                        #existing_struct_fields
                        #(#struct_fields_new),*
                    }
                }
        }
    );

    let mut existing_generics_impl = existing_generics.clone();
    existing_generics_impl.iter_mut().for_each(|g| match g {
        syn::GenericParam::Lifetime(_) => todo!(),
        syn::GenericParam::Type(ty) => ty.bounds.push(syn::TypeParamBound::Trait(
            syn::parse(quote!(Clone).into()).unwrap(),
        )),
        syn::GenericParam::Const(_) => todo!(),
    });

    quote!(
        #new_struct
        #new
    )
    .into()
}

fn quote_struct(component: &ComponentStruct, item_struct: &ItemStruct) -> proc_macro2::TokenStream {
    let attrs = &item_struct.attrs;
    let visibility = &item_struct.vis;
    let id = &item_struct.ident;
    let existing_generics = &item_struct.generics.params;
    let generics = &component.generics();

    let mut where_clause = quote!(where);
    if let Some(ref wher) = item_struct.generics.where_clause {
        let wher = &wher.predicates;
        where_clause.extend(quote!( #wher ))
    }

    let existing_fields = item_struct.fields.iter().collect::<Vec<_>>();
    let fields = component.driver_and_contract_fields();

    quote!(
        #(#attrs),*
        #visibility struct #id <#( #generics ),*, #existing_generics> #where_clause {
            #fields
            #( #existing_fields ),*
        }
    )
}

#[derive(Debug, Default)]
struct ComponentImpl {
    drivers: Vec<DriverImpl>,
    contracts: Vec<ContractImpl>,
}

impl ComponentImpl {
    fn impl_generics(&self) -> Vec<Ident> {
        let mut generics: Vec<_> = self.drivers.iter().map(|d| d.driver_id.clone()).collect();
        let conns = self.drivers.iter().map(|d| d.conn_id.clone());
        let contracts = self.contracts.iter().map(|d| d.name.clone());

        generics.extend(conns);
        generics.extend(contracts);

        generics
    }

    fn ty_generics(&self) -> Vec<Ident> {
        let mut generics: Vec<_> = self.drivers.iter().map(|d| d.driver_id.clone()).collect();
        let contracts = self.contracts.iter().map(|d| d.name.clone());

        generics.extend(contracts);

        generics
    }

    fn extend_where_clause(&self, where_clause: &mut proc_macro2::TokenStream) {
        let mut atomic_conns = Vec::new();

        for driver in self.drivers.iter() {
            let id = &driver.driver_id;
            let conn = &driver.conn_id;
            let atomic = driver.atomic.then_some(quote!(hextacy::Atomic+));
            if driver.atomic {
                atomic_conns.push(conn.clone());
            }
            where_clause.extend(quote!(
                #conn: #atomic Send,
                #id: hextacy::Driver<Connection = #conn> + Send + Sync,
            ));
        }

        for contract in self.contracts.iter() {
            let name = &contract.name;
            let trait_id = &contract.trait_id;
            let conn = &contract.conn_id;
            let atomic = atomic_conns
                .contains(conn)
                .then_some(quote!( + #trait_id<#conn::TransactionResult>));
            where_clause.extend(quote!(
                #name: #trait_id<#conn> #atomic + Send + Sync,
            ));
        }
    }
}

impl Parse for ComponentImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Self::default();

        while input.parse::<Token!(use)>().is_ok() {
            // Checks for 'with' or a fully qualified path
            if input.peek2(Ident) || input.peek2(Token!(::)) {
                let contract = input.parse()?;
                this.contracts.push(contract);
            }

            if input.peek2(Token!(for)) {
                let driver = input.parse()?;
                this.drivers.push(driver);
            }
        }
        Ok(this)
    }
}

#[derive(Debug)]
struct DriverImpl {
    driver_id: Ident,
    conn_id: Ident,
    atomic: bool,
}

impl Parse for DriverImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let driver_id: Ident = input.parse()?;
        input.parse::<Token!(for)>()?;
        let conn_id: Ident = input.parse()?;

        let mut this = DriverImpl {
            driver_id,
            conn_id: conn_id.clone(),
            atomic: false,
        };

        if input.peek(Token!(,)) {
            input.parse::<Token!(,)>().unwrap();
            return Ok(this);
        }

        if input.peek(Token!(:)) {
            input.parse::<Token!(:)>().unwrap();
            if let Ok(id) = input.parse::<Ident>() {
                if id == "Atomic" {
                    this.atomic = true;
                } else {
                    return Err(syn::Error::new(id.span(), "Invalid connection modifier"));
                }
            }
        }

        if input.peek(Token!(,)) {
            input.parse::<Token!(,)>().unwrap();
            return Ok(this);
        }

        match input.parse::<Token!(,)>() {
            Ok(_) => Ok(this),
            Err(_) => Ok(this),
        }
    }
}

#[derive(Debug)]
struct ContractImpl {
    trait_id: syn::Path,
    conn_id: Ident,
    name: Ident,
}

impl Parse for ContractImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_id = input.parse()?;
        let with = input.parse::<Ident>()?;
        if with != "with" {
            return Err(syn::Error::new(with.span(), "Expected token 'with'"));
        }
        let conn_id = input.parse()?;
        input.parse::<Token!(as)>()?;
        let name = input.parse()?;
        let this = Self {
            trait_id,
            conn_id,
            name,
        };

        match input.parse::<Token!(,)>() {
            Ok(_) => Ok(this),
            Err(_) => Ok(this),
        }
    }
}

#[derive(Debug, Default)]
struct ComponentStruct {
    drivers: Vec<DriverStruct>,
    generics: Vec<Ident>,
}

impl ComponentStruct {
    fn generics(&self) -> Vec<Ident> {
        let mut generics: Vec<_> = self.drivers.iter().map(|d| d.driver_id.clone()).collect();
        let gens = self.generics.iter().cloned();
        generics.extend(gens);

        generics
    }

    fn driver_and_contract_fields(&self) -> proc_macro2::TokenStream {
        let mut tokens = quote!();
        for driver in self.drivers.iter() {
            let field = Ident::new(
                &pascal_to_snake(&driver.name.to_string()),
                Span::call_site(),
            );
            let name = &driver.driver_id;
            tokens.extend(quote!(
                #field: #name,
            ));
        }

        for generic in self.generics.iter() {
            let field = Ident::new(&pascal_to_snake(&generic.to_string()), Span::call_site());
            tokens.extend(quote!(
                #field: #generic,
            ));
        }

        tokens
    }

    /// Get the necessary struct fields for implementing new
    fn driver_contract_fields_new(&self) -> Vec<Ident> {
        let mut tokens = vec![];
        tokens.extend(
            self.drivers
                .iter()
                .map(|f| Ident::new(&pascal_to_snake(&f.name.to_string()), Span::call_site())),
        );
        tokens.extend(
            self.generics
                .iter()
                .map(|f| Ident::new(&pascal_to_snake(&f.to_string()), Span::call_site())),
        );

        tokens
    }
}

impl Parse for ComponentStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut this = Self::default();

        while input.parse::<Token!(use)>().is_ok() {
            if input.peek2(Token!(as)) {
                let driver = input.parse()?;
                this.drivers.push(driver);
            } else {
                while let Ok(generic) = input.parse::<Ident>() {
                    this.generics.push(generic);
                    if input.parse::<Token!(,)>().is_ok() && input.cursor().eof() {
                        return Ok(this);
                    }
                }
            }
        }
        Ok(this)
    }
}

#[derive(Debug)]
struct DriverStruct {
    driver_id: Ident,
    name: Ident,
}

impl Parse for DriverStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let driver_id: Ident = input.parse()?;
        input.parse::<Token!(as)>()?;
        let name = input.parse()?;

        let this = DriverStruct { driver_id, name };

        match input.parse::<Token!(,)>() {
            Ok(_) => Ok(this),
            Err(_) => Ok(this),
        }
    }
}

fn pascal_to_snake(pascal_string: &str) -> String {
    pascal_string
        .chars()
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if i > 0 && c.is_uppercase() {
                acc.push('_');
            }
            acc.push(c.to_lowercase().next().unwrap());
            acc
        })
}
