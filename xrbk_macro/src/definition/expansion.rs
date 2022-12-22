// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod datasize;
mod readable;
mod r#trait;
mod writable;

use super::*;
use proc_macro2::TokenStream;
use quote::ToTokens;

impl ToTokens for Definitions {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self(definitions) = self;

		for definition in definitions {
			definition.to_tokens(tokens);
		}
	}
}

impl ToTokens for Definition {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Structlike(metadata, content, semicolon) => {
				metadata.to_tokens(tokens);
				content.to_tokens(tokens);
				semicolon.to_tokens(tokens);

				match metadata {
					Metadata::Request(request) => request.impl_trait(tokens),
					Metadata::Reply(reply) => reply.impl_trait(tokens, content),
					Metadata::Event(event) => event.impl_trait(tokens, content),

					_ => (),
				}

				metadata.impl_writable(tokens, content);
				metadata.impl_readable(tokens, content);
				metadata.impl_datasize(tokens, content);
			},

			Self::Enum(r#enum) => {
				r#enum.to_tokens(tokens);

				r#enum.impl_writable(tokens);
				r#enum.impl_readable(tokens);
				// TODO: r#enum.impl_datasize(tokens);
			},

			Self::Other(item) => item.to_tokens(tokens),
		}
	}
}

impl ToTokens for Metadata {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let (attributes, visibility, struct_token, ident, generics) = match self {
			Self::Struct(r#struct) => (
				&r#struct.attributes,
				&r#struct.visibility,
				&r#struct.struct_token,
				&r#struct.ident,
				&r#struct.generics,
			),

			Self::Request(request) => (
				&request.attributes,
				&request.visibility,
				&request.struct_token,
				&request.ident,
				&request.generics,
			),

			Self::Reply(reply) => (
				&reply.attributes,
				&reply.visibility,
				&reply.struct_token,
				&reply.ident,
				&reply.generics,
			),

			Self::Event(event) => (
				&event.attributes,
				&event.visibility,
				&event.struct_token,
				&event.ident,
				&event.generics,
			),
		};

		for attribute in attributes {
			attribute.to_tokens(tokens);
		}

		visibility.to_tokens(tokens);
		struct_token.to_tokens(tokens);
		ident.to_tokens(tokens);
		generics.to_tokens(tokens);
	}
}

impl ToTokens for Enum {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		for attribute in &self.attributes {
			attribute.to_tokens(tokens);
		}

		self.visibility.to_tokens(tokens);
		self.enum_token.to_tokens(tokens);
		self.ident.to_tokens(tokens);
		self.generics.to_tokens(tokens);

		self.brace_token.surround(tokens, |tokens| {
			self.variants.to_tokens(tokens);
		});
	}
}

impl ToTokens for Variant {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		for attribute in &self.attributes {
			attribute.to_tokens(tokens);
		}

		self.ident.to_tokens(tokens);
		self.content.to_tokens(tokens);

		if let Some((equals, expr)) = self.discriminant.as_ref() {
			equals.to_tokens(tokens);
			expr.to_tokens(tokens);
		}
	}
}