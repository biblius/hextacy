use quote::quote;
use syn::DeriveInput;

pub fn impl_response(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ident = &input.ident;
    let (im, ty, wh) = input.generics.split_for_impl();

    for attr in input.attrs.iter() {
        attr.meta.path().is_ident(ident);
    }

    Ok(quote!(
        impl #im Response<'_> for #ident #ty #wh {}
    ))
}
