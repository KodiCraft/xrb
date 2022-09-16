// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod content;
mod message;
mod util;

use util::*;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::quote;

use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, DeriveInput, Result};

use crate::message::*;

struct Messages {
	pub messages: Vec<Message>,
}

// TODO: Define `Definition` (in its own module). Used for simple enum and
// struct definitions.
struct Definition;

struct Definitions {
	pub definitions: Vec<Definition>,
}

impl Parse for Messages {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut messages: Vec<Message> = vec![];

		while !input.is_empty() {
			messages.push(input.parse()?);
		}

		Ok(Self { messages })
	}
}

impl Parse for Definitions {
	fn parse(_input: ParseStream) -> Result<Self> {
		// TODO: Parse definitions.
		Ok(Self {
			definitions: vec![],
		})
	}
}

/// Defines `struct`s for X11 protocol messages and automatically generates
/// trait implementations.
///
/// Specifically, those trait implementations include the trait relevant for
/// that particular message (`crate::Request`, `crate::Reply`, or
/// `crate::Event`), as well as for serialization and deserialization with
/// `cornflakes::ToBytes` and `cornflakes::FromBytes`, respectively.
#[proc_macro]
pub fn messages(input: TokenStream) -> TokenStream {
	// Parse the input as a stream of [`Messages`].
	let input = parse_macro_input!(input as Messages);

	// The list of messages.
	let messages = input.messages;

	// The trait implementations, not including serialization and deserialization.
	let trait_impls: Vec<TokenStream2> = messages
		.iter()
		.map(|message| message.message_trait_impl())
		.collect();

	let expanded = quote! {
		#(#messages)*
		#(#trait_impls)*
	};

	expanded.into()
}

/// Defines enums and structs with special syntax to generate their
/// (de)serialization.
///
/// This uses the same syntax as [`messages!`].
///
/// [`messages!`]: messages
#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
	// Parse the input as a stream of [`Definitions`].
	let input = parse_macro_input!(input as Definitions);

	// The list of definitions.
	let _definitions = input.definitions;

	let expanded = quote! {
		// TODO
	};

	expanded.into()
}

/// Derives an implementation of `cornflakes::ByteSize` for an enum or struct.
#[proc_macro_derive(ByteSize)]
pub fn derive_byte_size(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	let generics = add_trait_bounds(input.generics, quote!(cornflakes::ByteSize));
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let sum = byte_size_sum(&input.data);

	let expanded = quote! {
		impl #impl_generics cornflakes::ByteSize for #name #ty_generics #where_clause {
			fn byte_size(&self) -> usize {
				#sum
			}
		}
	};

	expanded.into()
}

/// Derives an implementation of `cornflakes::Writable` for an enum or struct.
#[proc_macro_derive(Writable)]
pub fn derive_writable(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	let generics = add_trait_bounds(input.generics, quote!(cornflakes::Writable));
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	// let write = writable_write(&input.data);

	let expanded = quote! {
		impl #impl_generics cornflakes::Writable for #name #ty_generics #where_clause {
			fn write_to(
				&self,
				writer: &mut impl cornflakes::Writer,
			) -> Result<(), cornflakes::WriteError> {
				// #write

				Ok(())
			}
		}
	};

	expanded.into()
}

/// Derives an implementation of `cornflakes::Readable` for an enum or struct.
#[proc_macro_derive(Readable)]
pub fn derive_readable(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = input.ident;

	let generics = add_trait_bounds(input.generics, quote!(cornflakes::Readable));
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	// let read = readable_impl(&input.data);

	let expanded = quote! {
		impl #impl_generics cornflakes::Readable for #name #ty_generics #where_clause {
			fn read_from(reader: &mut impl cornflakes::Reader) -> Result<Self, cornflakes::ReadError> {
				// #read
			}
		}
	};

	expanded.into()
}
