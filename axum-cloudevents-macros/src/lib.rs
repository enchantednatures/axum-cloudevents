//! Procedural macros for
//! [`axum-cloudevents`](https://github.com/enchantednatures/axum-cloudevents).
//!
//! The [`CloudEventTagged`] derive generates an inherent
//! `from_cloud_event_parts(event_type, data)` helper for enums. It does **not**
//! implement `Deserialize` and does not hook into the extractor automatically:
//! extract a `CloudEvent<serde_json::Value>` and call the generated method
//! with the event type and data.
//!
//! # Attributes
//!
//! - `#[cloud_event_prefix = "com.example.thing"]` on the enum: optional prefix.
//! - `#[cloud_event_type("created")]` on each variant: required event type
//!   suffix.
//!
//! The generated method accepts either the fully prefixed type
//! (`com.example.thing.created`) or the bare suffix (`created`) and returns
//! `Result<Self, String>`.
//!
//! # Example
//!
//! ```ignore
//! use axum_cloudevents::CloudEventTagged;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct Created {
//!     id: String,
//! }
//!
//! #[derive(Debug, CloudEventTagged)]
//! #[cloud_event_prefix = "com.example.thing"]
//! enum ThingEvent {
//!     #[cloud_event_type("created")]
//!     Created(Created),
//! }
//!
//! let event = ThingEvent::from_cloud_event_parts(
//!     "com.example.thing.created",
//!     serde_json::json!({ "id": "thing-1" }),
//! )
//! .expect("known event type");
//! ```

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprLit, Lit, Meta};

/// Read a `#[attr = "value"]` or `#[attr("value")]` string attribute.
fn string_attr(attrs: &[syn::Attribute], ident: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident(ident) {
            return None;
        }
        match &attr.meta {
            Meta::NameValue(nv) => match &nv.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) => Some(s.value()),
                _ => None,
            },
            Meta::List(list) => list.parse_args::<syn::LitStr>().ok().map(|s| s.value()),
            Meta::Path(_) => None,
        }
    })
}

/// Derives `from_cloud_event_parts` for a CloudEvents enum.
///
/// The enum must be an enum; every variant needs a
/// `#[cloud_event_type("...")]` attribute. The enum may optionally declare
/// `#[cloud_event_prefix = "..."]`.
#[proc_macro_derive(CloudEventTagged, attributes(cloud_event_prefix, cloud_event_type))]
pub fn derive_cloud_event_tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let enum_data = match &input.data {
        Data::Enum(e) => &e.variants,
        _ => panic!("CloudEventTagged can only be derived for enums"),
    };

    let prefix = string_attr(&input.attrs, "cloud_event_prefix").unwrap_or_default();

    let mut variant_match_arms = Vec::new();
    let mut variant_types = Vec::new();

    for variant in enum_data {
        let variant_name = &variant.ident;

        let event_type = string_attr(&variant.attrs, "cloud_event_type").unwrap_or_else(|| {
            panic!(
                "Variant {} missing #[cloud_event_type(\"...\")]",
                variant_name
            )
        });

        variant_types.push(variant_name.clone());

        variant_match_arms.push(quote! {
            #event_type => {
                let payload = serde_json::from_value(data)
                    .map_err(|e| e.to_string())?;
                Ok(Self::#variant_name(payload))
            }
        });
    }

    let prefix_str = prefix.as_str();

    let expanded = quote! {
        impl #name {
            pub fn from_cloud_event_parts(
                event_type: &str,
                data: serde_json::Value,
            ) -> Result<Self, String> {
                let suffix = event_type
                    .strip_prefix(#prefix_str)
                    .and_then(|s| s.strip_prefix('.'))
                    .unwrap_or(event_type);

                match suffix {
                    #(#variant_match_arms)*
                    _ => Err(format!("Unknown event type: {}", event_type))
                }
            }
        }
    };

    TokenStream::from(expanded)
}
