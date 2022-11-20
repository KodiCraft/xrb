// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
	braced, bracketed, parenthesized,
	parse::{Parse, ParseStream, Result},
	punctuated::{Pair, Punctuated},
	spanned::Spanned,
	token, Error, Ident, Token,
};

use crate::*;

mod iter;

pub enum Items {
	/// [`Item`]s surrounded by curly brackets (`{` and `}`), with names for
	/// [`Field`]s.
	Named {
		brace_token: token::Brace,
		items: Punctuated<ItemWithId, Token![,]>,
	},

	/// [`Item`]s surrounded by normal brackets (`(` and `)`), without names
	/// for [`Field`]s.
	Unnamed {
		paren_token: token::Paren,
		items: Punctuated<ItemWithId, Token![,]>,
	},

	/// No [`Item`]s at all.
	Unit,
}

type ItemWithId = (ItemId, Item);

pub enum ItemId {
	/// An `ItemId` associated with fields.
	///
	/// Named fields have a [`FieldId::Ident`] ID, unnamed fields have a
	/// [`FieldId::Id`] ID.
	Field(FieldId),

	/// An `ItemId` associated with unused bytes items.
	///
	/// 'Array-type' unused bytes items have a `usize` ID, 'unit-type' unused
	/// bytes items do not.
	Unused(Option<usize>),

	/// An `ItemId` associated with let-items.
	Let(Ident),
}

pub enum FieldId {
	Ident(Ident),
	Id(usize),
}

impl ItemId {
	pub fn formatted(&self) -> Option<Ident> {
		match self {
			Self::Field(id) => Some(id.formatted()),

			Self::Unused(id) => id.map(|id| format_ident!("_unused_{}_", id)),

			Self::Let(id) => Some(format_ident!("__{}__", id)),
		}
	}
}

impl FieldId {
	pub fn formatted(&self) -> Ident {
		match self {
			Self::Ident(id) => format_ident!("__{}__", id),
			Self::Id(id) => format_ident!("__{}__", id),
		}
	}
}

// Expansion {{{

impl ToTokens for Items {
	fn to_tokens(&self, tokens: &mut TokenStream2) {
		/// An internal-use function within `to_tokens` to reduce repeated
		/// code. This ensures that commas are only converted to tokens if
		/// their respective item is.
		fn items_to_tokens(items: &Punctuated<ItemWithId, Token![,]>, tokens: &mut TokenStream2) {
			// For every pair of item and a possible comma...
			for pair in items.pairs() {
				// Unwrap the item and comma (which will be `None` if it is the
				// final item and there is no trailing comma).
				let (item, comma) = match pair {
					Pair::Punctuated((_, item), comma) => (item, Some(comma)),
					Pair::End((_, item)) => (item, None),
				};

				// If this is a field, convert the  field and the comma to
				// tokens, otherwise... don't.
				if let Item::Field(field) = item {
					field.to_tokens(tokens);
					comma.to_tokens(tokens);
				}
			}
		}

		match self {
			// Surround named items with their curly brackets.
			Self::Named { brace_token, items } => {
				brace_token.surround(tokens, |tokens| items_to_tokens(items, tokens));
			}

			// Surround unnamed items with their normal brackets.
			Self::Unnamed { paren_token, items } => {
				paren_token.surround(tokens, |tokens| items_to_tokens(items, tokens));
			}

			// Don't convert `Self::Unit` to any tokens at all.
			Self::Unit => (),
		}
	}
}

pub enum ExpandMode {
	Normal,
	Request,

	Reply { has_sequence: bool },
	Event,
}

impl Items {
	/// Generates the pattern required to pattern-match against these items
	/// (e.g. in a `match` expression).
	pub fn pattern_to_tokens(&self, tokens: &mut TokenStream2, mode: ExpandMode) {
		/// An internal-use function within `patterns_to_tokens` to reduce
		/// repeated code. This generates the pattern to match against the
		/// items.
		fn pattern(tokens: &mut TokenStream2, items: &Items, mode: ExpandMode) {
			match mode {
				ExpandMode::Normal => {}
				ExpandMode::Request => {}

				ExpandMode::Reply { has_sequence } => {
					if has_sequence {
						tokens.append_tokens(|| quote!(_sequence_,));
					}
				}
				ExpandMode::Event => tokens.append_tokens(|| quote!(_sequence_,)),
			}

			for (id, _) in items.pairs() {
				// Only generate the pattern for fields.
				if let ItemId::Field(field_id) = id {
					if let FieldId::Ident(ident) = field_id {
						tokens.append_tokens(|| quote!(#ident: ));
					}

					// Convert the field's formatted identifier to tokens.
					id.formatted().to_tokens(tokens);

					// Append a comma too.
					tokens.append_tokens(|| quote!(,));
				}
			}
		}

		match self {
			// Surround named item patterns with their curly brackets.
			Self::Named { brace_token, .. } => {
				brace_token.surround(tokens, |tokens| pattern(tokens, self, mode));
			}

			// Surround unnamed items with their normal brackets.
			Self::Unnamed { paren_token, .. } => {
				paren_token.surround(tokens, |tokens| pattern(tokens, self, mode))
			}

			// Don't generate a pattern for `Self::Unit` at all.
			Self::Unit => {}
		}
	}

	/// Generates the tokens required to construct the struct or enum variant
	/// using these `Items`.
	pub fn constructor_to_tokens(&self, tokens: &mut TokenStream2) {
		match self {
			Self::Unit => {}

			Self::Named { brace_token, .. } => {
				brace_token.surround(tokens, |tokens| {
					for (id, _) in self.pairs() {
						if let ItemId::Field(FieldId::Ident(name)) = id {
							let val = id.formatted();

							tokens.append_tokens(|| quote!(#name: #val,));
						}
					}
				});
			}

			Self::Unnamed { paren_token, .. } => {
				paren_token.surround(tokens, |tokens| {
					for (id, _) in self.pairs() {
						let val = id.formatted();

						tokens.append_tokens(|| quote!(#val,));
					}
				});
			}
		}
	}
}

// }}}

// Parsing {{{

impl Items {
	pub(self) fn parse_items(
		input: ParseStream,
		named: bool,
	) -> Result<Punctuated<ItemWithId, Token![,]>> {
		let mut unused_index: usize = 0;
		let mut field_index: usize = 0;

		let mut items = Punctuated::new();
		// Keep track of the identifiers defined thus far and which types they
		// correspond to. This is used to parse `Source`s.
		let mut map = HashMap::new();

		// While there are still tokens left in the `input` stream, we continue
		// to parse items.
		while !input.is_empty() {
			if input.peek(Token![#]) {
				let mut attributes = Attribute::parse_outer(input, &map)?;

				if input.peek(token::Bracket) || input.peek(token::Paren) {
					// Unused bytes item.

					if let Some(attr) = attributes.first() {
						if !attr.is_metabyte() {
							return Err(Error::new(
								attr.span(),
								"only a metabyte attribute is allowed for unused items",
							));
						}
					} else if let Some(attr) = attributes.get(1) {
						return Err(Error::new(
							attr.span(),
							"only zero or one (metabyte) attributes are allowed for unused items",
						));
					}

					let _unit;
					let (id, unused) = {
						if !attributes.is_empty() {
							// Unit with attribute.

							(
								ItemId::Unused(None),
								Unused::Unit {
									attribute: Some(attributes.remove(0)),
									unit_token: parenthesized!(_unit in input),
								},
							)
						} else if input.peek(token::Paren) {
							// Unit, no attribute.

							(
								ItemId::Unused(None),
								Unused::Unit {
									attribute: None,
									unit_token: parenthesized!(_unit in input),
								},
							)
						} else {
							// Array.

							let content;

							let index = unused_index;
							unused_index += 1;

							(
								ItemId::Unused(Some(index)),
								Unused::Array(Box::new(Array {
									bracket_token: bracketed!(content in input),
									unit_token: parenthesized!(_unit in content),
									semicolon_token: content.parse()?,
									source: Source::parse(input, &map)?,
								})),
							)
						}
					};

					items.push_value((id, Item::Unused(unused)));
				} else if input.peek(Token![let]) {
					// Let item.

					if let Some(attr) = attributes.first() {
						if !attr.is_metabyte() {
							return Err(Error::new(
								attr.span(),
								"only a metabyte attribute is allowed for let items",
							));
						}
					} else if let Some(attr) = attributes.get(1) {
						return Err(Error::new(
							attr.span(),
							"only zero or one (metabyte) attributes are allowed for let items",
						));
					}
				} else {
					// Field item.
				}
			}

			if input.peek(token::Bracket) || input.peek(token::Paren) {
				// If the next token (i.e. the start of a new item) is a square
				// bracket or a normal bracket, then this must be an unused
				// bytes item (either in the form `[(); source]`, or just `()`).

				let unused = Unused::parse(input, &map)?;

				let id = match unused {
					Unused::Array(_) => {
						// 'Save' the current `unused_index` to return it.
						let index = unused_index;

						// If this is an `Unused::Array`, it will use the
						// `unused_index`, which must therefore be incremented
						// by one:
						unused_index += 1;

						Some(index)
					}

					// `Unused::Unit` uses no index because it does not
					// generate a source function that is to be referred to.
					Unused::Unit { .. } => None,
				};

				items.push_value((ItemId::Unused(id), Item::Unused(unused)));
			} else if input.peek(Token![let]) {
				// Otherwise, if the next token is `Let`, then this must be a
				// `Let` item. Note that this won't work if support for
				// attributes is added to `Let` items: in that case we would
				// have to parse all of the attributes before we could work out
				// if it was a `Field` item or a `Let` item.
				let r#let: Let = input.parse()?;

				// We insert the name of the `Let` item into the `map`, since
				// it will be able to be referred to by name in `Source`s, and
				// we'll want to know its type.
				map.insert(r#let.ident.to_owned(), r#let.r#type.to_owned());

				// Push the new `Item::Let` to the list of `items`.
				items.push_value((
					ItemId::Let(r#let.ident.to_owned()),
					Item::Let(Box::new(r#let)),
				));
			} else {
				// Otherwise, if this is not an unused bytes item, nor a `Let`
				// item, we assume it is a `Field` and parse it accordingly.

				let field = if named {
					// If we are to parse the items as `named`, then we parse
					// the `field` as as named:
					Field::parse_named(input, &map)?
				} else {
					// Otherwise, we parse the field as unnamed:
					Field::parse_unnamed(input, &map)?
				};

				let id = if let Some(ident) = &field.ident {
					FieldId::Ident(ident.to_owned())
				} else {
					let index = field_index;
					field_index += 1;

					FieldId::Id(index)
				};

				match &id {
					FieldId::Ident(ident) => {
						map.insert(ident.to_owned(), field.r#type.to_owned());
					}

					FieldId::Id(id) => {
						map.insert(
							Ident::new(&id.to_string(), Span::call_site()),
							field.r#type.to_owned(),
						);
					}
				}

				// Then we push the `Item::Field` to the list of `items`.
				items.push_value((ItemId::Field(id), Item::Field(Box::new(field))));
			}

			// If the token following the item is not a comma, then it must be
			// the end of the list, so we break from the loop.
			if !input.peek(Token![,]) {
				break;
			} else {
				// Otherwise, if the next token is a comma, then the list can
				// continue: we add the comma to the list.
				items.push_punct(input.parse()?);
			}
		}

		Ok(items)
	}

	/// Parse [`Items`] surrounded by curly brackets (`{` and `}`) and with
	/// named [`Field`s`](Field).
	pub fn parse_named(input: ParseStream) -> Result<Self> {
		let content;

		let brace_token = braced!(content in input);
		let items = Self::parse_items(&content, true)?;

		Ok(Self::Named { brace_token, items })
	}

	/// Parse [`Items`] surrounded by normal brackets (`(` and `)`) and with
	/// unnamed [`Field`s](Field).
	pub fn parse_unnamed(input: ParseStream) -> Result<Self> {
		let content;

		let paren_token = parenthesized!(content in input);
		let items = Self::parse_items(&content, false)?;

		Ok(Self::Unnamed { paren_token, items })
	}
}

impl Parse for Items {
	fn parse(input: ParseStream) -> Result<Self> {
		if input.peek(token::Brace) {
			// If the next token is a curly bracket (`{`), parse as named
			// `Item`s.
			Self::parse_named(input)
		} else if input.peek(token::Paren) {
			// Otherwise, if the next token is a normal bracket (`(`), parse as
			// unnamed `Item`s.
			Self::parse_unnamed(input)
		} else {
			// Otherwise, if the next token is neither a curly bracket (`{`),
			// nor a normal bracket (`(`), there are no items; simply return
			// `Self::Unit`.
			Ok(Self::Unit)
		}
	}
}

// }}}
