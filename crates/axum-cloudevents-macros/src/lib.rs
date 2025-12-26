use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum, Variant, Attribute, Lit};
use proc_macro2::{TokenStream as TokenStream2, TokenTree};

#[proc_macro_derive(CloudEventTagged, attributes(cloud_event_prefix, cloud_event_variant, cloud_event_catchall))]
pub fn derive_cloud_event_tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (prefix, variants) = parse_tagged_input(&input);

    let match_arms: Vec<proc_macro2::TokenStream> = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let event_type_suffix = variant
                .event_type_suffix
                .as_ref()
                .unwrap_or_else(|| {
                    panic!("Variant {} missing #[cloud_event_variant(\"...\")]", variant_name)
                });

            let catchall = variant.catchall;
            let event_type_check = if catchall {
                quote! { _ }
            } else {
                quote! { #event_type_suffix }
            };

            quote! {
                #event_type_check => {
                    let payload = serde_json::from_value(data)
                        .map_err(|e| e.to_string())?;
                    Ok(Self::#variant_name(payload))
                }
            }
        })
        .collect();

    let prefix_str = prefix.as_str();

    let expanded = quote! {
        impl axum_cloudevents_tagged::CloudEventTagged for #name {
            const TYPE_PREFIX: &'static str = #prefix_str;

            fn from_cloud_event_parts(event_type: &str, data: serde_json::Value) -> Result<Self, String> {
                let suffix = event_type
                    .strip_prefix(#prefix_str)
                    .and_then(|s| s.strip_prefix('.'))
                    .unwrap_or(event_type);

                match suffix {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IntoCloudEventResponse, attributes(cloud_event))]
pub fn derive_into_cloud_event_response(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut event_type = None;
    let mut event_source = None;

    for attr in &input.attrs {
        if attr.path().is_ident("cloud_event") {
            match parse_cloud_event_attr(attr) {
                Ok(parsed) => {
                    event_type = parsed.type_value;
                    event_source = parsed.source_value;
                }
                Err(e) => {
                    panic!("Failed to parse cloud_event attribute: {}", e);
                }
            }
        }
    }

    let event_type = event_type.unwrap_or_else(|| {
        panic!(
            "Missing #[cloud_event(type = \"...\")] attribute on {}",
            name
        )
    });
    let event_source = event_source.unwrap_or_else(|| "/axum-cloudevents".to_string());

    let expanded = quote! {
        impl axum_cloudevents_core::IntoCloudEventResponse for #name {
            fn cloud_event_type() -> &'static str {
                #event_type
            }

            fn cloud_event_source() -> &'static str {
                #event_source
            }

            fn into_cloud_event(self) -> axum_cloudevents_core::CloudEvent<Self>
            where
                Self: Sized,
            {
                let id = self.cloud_event_id();
                let time = self.cloud_event_time();

                let mut metadata = axum_cloudevents_core::CloudEventMetadata::new(
                    id,
                    Self::cloud_event_source(),
                    Self::cloud_event_type()
                );

                if let Some(t) = time {
                    metadata.time = Some(t);
                }

                axum_cloudevents_core::CloudEvent::new(metadata, self)
            }
        }

        impl axum::response::IntoResponse for #name
        where
            Self: serde::Serialize,
        {
            fn into_response(self) -> axum::response::Response {
                self.into_cloud_event().into_response()
            }
        }
    };

    TokenStream::from(expanded)
}

struct CloudEventAttr {
    type_value: Option<String>,
    source_value: Option<String>,
}

fn parse_cloud_event_attr(attr: &Attribute) -> syn::Result<CloudEventAttr, syn::Error> {
    let tokens = &attr.meta.require_list()?.tokens;
    let mut type_value = None;
    let mut source_value = None;
    let mut iter = tokens.clone().into_iter().peekable();

    while iter.peek().is_some() {
        if let Some(TokenTree::Ident(ident)) = iter.next() {
            if iter.peek().is_some() {
                let _ = iter.next(); // skip equals
            }
            if let Some(lit) = iter.next() {
                if ident.to_string() == "type" {
                    type_value = Some(get_lit_str(&lit)?);
                } else if ident.to_string() == "source" {
                    source_value = Some(get_lit_str(&lit)?);
                }
            }
        } else {
            let _ = iter.next(); // skip unknown token
        }

        if iter.peek().is_some() {
            let _ = iter.next(); // skip comma
        }
    }

    Ok(CloudEventAttr {
        type_value,
        source_value,
    })
}

fn get_lit_str(node: &TokenTree) -> syn::Result<String, syn::Error> {
    match node {
        TokenTree::Literal(lit) => {
            match lit {
                Lit::Str(s) => Ok(s.value()),
                _ => Err(syn::Error::new_spanned(
                    lit.span(),
                    syn::error::Error::new("expected string literal", lit.span()),
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            node.span(),
            syn::error::Error::new("expected literal", node.span()),
        )),
    }
}

fn parse_prefix_attr(attr: &Attribute) -> syn::Result<String, syn::Error> {
    let tokens = &attr.meta.require_list()?.tokens;
    
    if let Some(node) = tokens.first() {
        return Ok(get_lit_str(node)?);
    }
    
    Err(syn::Error::new_spanned(
        attr.span(),
        syn::error::Error::new("cloud_event_prefix requires a string literal", attr.span()),
    ))
}

fn parse_variant_attr(attr: &Attribute) -> syn::Result<String, syn::Error> {
    let tokens = &attr.meta.require_list()?.tokens;
    
    if let Some(node) = tokens.first() {
        return Ok(get_lit_str(node)?);
    }
    
    Err(syn::Error::new_spanned(
        attr.span(),
        syn::error::Error::new("cloud_event_variant requires a string literal", attr.span()),
    ))
}

struct VariantInfo {
    ident: syn::Ident,
    event_type_suffix: Option<String>,
    catchall: bool,
}

fn parse_tagged_input(input: &DeriveInput) -> (String, Vec<VariantInfo>) {
    let mut prefix = String::new();

    for attr in &input.attrs {
        if attr.path().is_ident("cloud_event_prefix") {
            match parse_prefix_attr(attr) {
                Ok(p) => prefix = p,
                Err(e) => panic!("Failed to parse cloud_event_prefix attribute: {}", e),
            }
        }
    }

    let variants = match &input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("CloudEventTagged can only be derived for enums"),
    };

    let variant_infos: Vec<VariantInfo> = variants
        .iter()
        .map(|variant| parse_variant(variant))
        .collect();

    (prefix, variant_infos)
}

fn parse_variant(variant: &Variant) -> VariantInfo {
    let mut event_type_suffix = None;
    let mut catchall = false;

    for attr in &variant.attrs {
        if attr.path().is_ident("cloud_event_variant") {
            match parse_variant_attr(attr) {
                Ok(s) => event_type_suffix = Some(s),
                Err(e) => panic!("Failed to parse cloud_event_variant attribute: {}", e),
            }
        } else if attr.path().is_ident("cloud_event_catchall") {
            catchall = true;
        }
    }

    if catchall {
        event_type_suffix = None;
    }

    VariantInfo {
        ident: variant.ident.clone(),
        event_type_suffix,
        catchall,
    }
}
