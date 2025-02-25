// devela_macros::bodies::niche
//
//! Bodies related to niche optimizations.
//
// TOC
// - enumint

use super::shared::{expect_punct, parse_int, parse_visibility};
use alloc::{format, string::ToString, vec::Vec};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

#[cfg(feature = "alloc")]
pub(crate) fn body_enumint(input: TokenStream) -> TokenStream {
    let input: TokenStream2 = input.into();
    let tokens: Vec<TokenTree> = input.into_iter().collect();

    // Process the tokens to extract: visibility, enum_name, repr, start, end
    // Expected format: [visibility] enum_name, repr, start, end
    let mut iter = tokens.into_iter().peekable();

    let visibility = parse_visibility(&mut iter);

    let enum_name = match iter.next() {
        Some(TokenTree::Ident(ident)) => ident,
        other => panic!("Expected enum name, found {:?}", other),
    };
    expect_punct(&mut iter, ',');

    let repr = match iter.next() {
        Some(TokenTree::Ident(ident)) => ident.to_string(),
        other => panic!("Expected representation type, found {:?}", other),
    };
    let repr_str = repr.as_str();

    expect_punct(&mut iter, ',');
    let start = parse_int(&mut iter);
    expect_punct(&mut iter, ',');
    let end = parse_int(&mut iter);

    // Ensure no more tokens
    if let Some(tok) = iter.next() {
        panic!("Unexpected token after end value: {:?}", tok);
    }

    // Validate the provided representation against the range length
    let range_length = end - start + 1;
    let repr = match repr_str {
        // unsigned reprs
        "u8" => {
            if range_length > u8::MAX as i128 {
                panic!("u8 cannot represent the range [{start}, {end}]")
            }
            quote! { u8 }
        }
        "u16" => {
            if range_length > u16::MAX as i128 {
                panic!("u16 cannot represent the range [{start}, {end}]")
            }
            quote! { u16 }
        }
        // signed reprs
        "i8" => {
            if start < i8::MIN as i128 || end > i8::MAX as i128 {
                panic!("i8 cannot represent the range [{start}, {end}]")
            }
            quote! { i8 }
        }
        "i16" => {
            if start < i16::MIN as i128 || end > i16::MAX as i128 {
                panic!("i16 cannot represent the range [{start}, {end}]")
            }
            quote! { i16 }
        }
        _ => panic!("Invalid representation type: {}", repr_str),
    };
    let unsigned_repr = match repr_str {
        "i8" => quote! { u8 },
        "i16" => quote! { u16 },
        _ => repr.clone(), // For unsigned types, use the same repr
    };

    let visibility = visibility.unwrap_or_default();
    let enum_name = Ident::new(&enum_name.to_string(), Span::call_site());

    // Generate the enum variants, handling negative and positive values.
    let mut enum_variants = Vec::new();
    for i in start..=end {
        let variant = if i < 0 {
            Ident::new(&format!("N{}", i.abs()), Span::call_site())
        } else {
            Ident::new(&format!("P{}", i), Span::call_site())
        };
        enum_variants.push(quote! { #variant = #i as #repr });
    }

    // Generate the final output
    let enum_definition = quote! {
        /// An auto-generated enum for values between #start and #end.
        #[allow(missing_docs)] // reason = "undocumented variants"
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(#repr)]
        #visibility enum #enum_name {
            #(
                #enum_variants
            ),*
        }

        // SAFETY: The type is Copy.
        unsafe impl Send for #enum_name {}
        unsafe impl Sync for #enum_name {}

        impl #enum_name {
            /* constants */

            /// Returns the number of valid values, as an unsigned primitive.
            pub const VALID_VALUES: #unsigned_repr = #range_length as #unsigned_repr;

            /// Returns the number of invalid values, as an unsigned primitive.
            pub const NICHE_VALUES: #unsigned_repr = #unsigned_repr::MAX - Self::VALID_VALUES + 1;

            /// Returns the minimum possible value.
            pub const MIN: #repr = #start as #repr;

            /// Returns the maximum possible value.
            pub const MAX: #repr = #end as #repr;

            /* methods */

            /// Returns the appropriate variant from the given `value`.
            ///
            /// Returns `None` if it's out of range.
            #[must_use]
            pub const fn new(value: #repr) -> Option<Self> {
                if value >= #start as #repr && value <= #end as #repr {
                    // SAFETY: The check ensures that `value` is within the valid range,
                    // so the `transmute` will always produce a valid enum variant.
                    Some(unsafe { core::mem::transmute(value) })
                } else {
                    None
                }
            }

            /// Returns the appropriate variant if the given `value` is within bounds.
            ///
            /// # Panics
            /// Panics in debug if `value < #start || value > #end`.
            /// # Safety
            /// The given `value` must always be `value >= #start && value <= #end`.
            #[must_use]
            pub const unsafe fn new_unchecked(value: #repr) -> Self {
                debug_assert!(value >= #start as #repr && value <= #end as #repr, "Value out of range");
                // SAFETY: caller must ensure safety
                unsafe { core::mem::transmute(value) }
            }

            /// Returns the appropriate variant from the given `value`,
            /// saturating at the type bounds.
            #[must_use]
            pub const fn new_saturated(value: #repr) -> Self {
                // SAFETY: The `clamp` function ensures that the value is within the valid range,
                // so the `transmute` will always produce a valid enum variant.
                unsafe { core::mem::transmute(Self::clamp(value, #start as #repr, #end as #repr)) }
            }

            /// Returns the appropriate variant from the given `value`,
            /// wrapping around within the type bounds.
            #[must_use]
            pub const fn new_wrapped(value: #repr) -> Self {
                let range_size = (#end - #start + 1) as #repr;
                let wrapped_value = if value >= #start as #repr {
                    (value - #start as #repr) % range_size + #start as #repr  // Upward wrapping
                } else {
                    let diff = #start as #repr - value;
                    #end as #repr - ((diff - 1) % range_size)  // Downward wrapping
                };
                // SAFETY: The `wrapped_value` is guaranteed to be within the valid range,
                // so `transmute` will always produce a valid enum variant.
                unsafe { core::mem::transmute(wrapped_value) }
            }

            /// Cast the enum to its underlying representation.
            #[must_use]
            pub const fn get(self) -> #repr {
                self as #repr
            }

            /* helpers */

            const fn clamp(v: #repr, min: #repr, max: #repr) -> #repr {
                if v < min { min } else if v > max { max } else { v }
            }
            // const fn max(a: {repr}) -> {repr} {{ if a > b {{ a }} else {{ b }} }}
            // const fn min(a: {repr}) -> {repr} {{ if a < b {{ a }} else {{ b }} }}
        }
    };
    enum_definition.into()
}
