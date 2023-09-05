use proc_macro_error::abort;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, Ident, ItemImpl, Token, TypePath};

pub fn impl_component(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let component: ComponentImpl = syn::parse(attr).expect("Could not parse component");
    let original: ItemImpl = syn::parse(input.clone()).expect("Expected impl block");

    // TODO
    // Implement for struct instead of drive!
    // Support additional generics

    assert!(
        original.generics.where_clause.is_none() && original.generics.gt_token.is_none(),
        "Cannot apply component, generics already exist"
    );

    let original_fns = &original.items;
    let original_struct = match original.self_ty.as_ref() {
        syn::Type::Path(TypePath { ref path, .. }) => &path.segments[0].ident,
        _ => abort!(
            original.self_ty.span(),
            "component not supported for this type of impl"
        ),
    };

    let attrs = original.attrs;

    let mut generics: Vec<_> = component
        .drivers
        .iter()
        .map(|d| d.driver_id.clone())
        .collect();
    let conns = component.drivers.iter().map(|d| d.conn_id.clone());
    let contracts = component.contracts.iter().map(|d| d.name.clone());

    generics.extend(conns);
    generics.extend(contracts);

    let mut where_clause = quote!(where);

    let mut atomic_conns = Vec::new();

    for driver in component.drivers.iter() {
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

    for contract in component.contracts {
        let name = contract.name;
        let trait_id = contract.trait_id;
        let conn = contract.conn_id;
        let atomic = atomic_conns
            .contains(&conn)
            .then_some(quote!( + #trait_id<#conn::TransactionResult>));
        where_clause.extend(quote!(
            #name: #trait_id<#conn> #atomic + Send + Sync,
        ));
    }

    quote!(
        #(#attrs)*
        impl<#(#generics),*> #original_struct <#(#generics),*> #where_clause
        {
            #(#original_fns)*
        }
    )
    .into()
}

#[derive(Debug, Default)]
struct ComponentImpl {
    drivers: Vec<DriverImpl>,
    contracts: Vec<ContractImpl>,
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
    rename: Option<Ident>,
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
            rename: None,
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

        input.parse::<Token!(as)>()?;

        this.rename = Some(input.parse()?);

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
