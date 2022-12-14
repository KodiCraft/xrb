// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use syn::{braced, bracketed, parenthesized, parse::ParseStream, spanned::Spanned, Error, Result};

use super::*;
use crate::{
	attribute::parsing::ParsedAttributes,
	source::{
		parsing::{IdentMap, IdentMapMut},
		Args,
	},
	ParseWithContext,
	PsExt,
};

#[derive(Clone, Copy)]
pub enum ElementType {
	Named,
	Unnamed,
}

#[derive(Clone, Copy)]
pub enum DefinitionType {
	Basic,

	Request,
	Reply,
	Event,
}

impl DefinitionType {
	pub fn length_syntax(&self) -> bool {
		match self {
			Self::Request => true,
			Self::Reply => true,

			_ => false,
		}
	}
}

impl ParseWithContext for Content<'_> {
	type Context = DefinitionType;

	fn parse_with(input: ParseStream, definition_type: DefinitionType) -> Result<Self> {
		Ok(if input.peek(token::Brace) {
			let content;

			Self::Struct {
				brace_token: braced!(content in input),
				elements: content.parse_with((ElementType::Named, definition_type))?,
			}
		} else if input.peek(token::Paren) {
			let content;

			Self::Tuple {
				paren_token: parenthesized!(content in input),
				elements: content.parse_with((ElementType::Unnamed, definition_type))?,
			}
		} else {
			Self::Unit
		})
	}
}

impl ParseWithContext for Elements<'_> {
	type Context = (ElementType, DefinitionType);

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self>
	where
		Self: Sized,
	{
		let (element_type, definition_type) = context;

		let mut let_map = HashMap::new();
		let mut field_map = HashMap::new();

		let mut elements = Punctuated::new();
		let mut metabyte_element = None;
		let mut sequence_field = None;

		let mut field_index = 0;
		let mut unused_index = 0;

		while !input.is_empty() {
			let element: Element = input.parse_with((
				(field_index, unused_index),
				element_type,
				definition_type,
				(&mut let_map, &mut field_map),
			))?;

			if element.is_metabyte() {
				if metabyte_element.is_some() {
					return Err(Error::new(
						element.span(),
						"no more than one metabyte element is allowed per message",
					));
				}

				metabyte_element = Some(&element);
			}

			if let Element::Field(field) = &element && field.is_sequence() {
				if sequence_field.is_some() {
					return Err(Error::new(
						field.span(),
						"no more than one sequence field is allowed per message",
					));
				}

				sequence_field = Some(&**field);
			}

			match &element {
				Element::Field(_) => field_index += 1,
				Element::ArrayUnused(_) => unused_index += 1,
				_ => (),
			}

			elements.push_value(element);

			if input.peek(Token![,]) {
				elements.push_punct(input.parse()?);
			} else {
				break;
			}
		}

		for element in elements {
			if let Element::Let(r#let) = element {
				if let Some((Args { args, .. }, _)) = r#let.source.args {
					for mut arg in args {
						if arg.r#type.is_none() {
							if let Some(r#type) = field_map.get(&arg.ident.to_string()) {
								arg.r#type = Some(r#type.to_owned());
							} else {
								return Err(Error::new(
									arg.ident.span(),
									"unrecognized source argument identifier",
								));
							}
						}
					}
				}
			}
		}

		Ok(Self {
			elements,

			metabyte_element,
			sequence_field,
		})
	}
}

impl ParseWithContext for Element<'_> {
	type Context<'a> = (
		(usize, usize),
		ElementType,
		DefinitionType,
		(IdentMapMut<'a>, IdentMapMut<'a>),
	);

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self> {
		let ((field_index, unused_index), element_type, definition_type, (let_map, field_map)) =
			context;
		let parsed_attributes = input.parse_with(((&*let_map, &*field_map), definition_type))?;

		Ok(if input.peek(Token![_]) {
			Self::SingleUnused(input.parse_with(parsed_attributes)?)
		} else if input.peek(token::Bracket) {
			Self::ArrayUnused(Box::new(input.parse_with((
				unused_index,
				parsed_attributes,
				(&*let_map, &*field_map),
				definition_type,
			))?))
		} else if input.peek(Token![let]) {
			Self::Let(Box::new(input.parse_with((
				parsed_attributes,
				let_map,
				definition_type,
			))?))
		} else {
			Self::Field(Box::new(input.parse_with((
				field_index,
				element_type,
				parsed_attributes,
				field_map,
			))?))
		})
	}
}

impl ParseWithContext for SingleUnused {
	type Context = ParsedAttributes;

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self> {
		let ParsedAttributes {
			attributes,

			context_attribute,
			metabyte_attribute,
			sequence_attribute,
		} = context;

		if let Some(attribute) = attributes.first() {
			return Err(Error::new(
				attribute.span(),
				"normal attributes are not allowed for singular unused bytes elements",
			));
		}

		if let Some(attribute) = context_attribute {
			return Err(Error::new(
				attribute.span(),
				"context attributes are not allowed for singular unused bytes elements",
			));
		}

		if let Some(attribute) = sequence_attribute {
			return Err(Error::new(
				attribute.span(),
				"sequence attributes are not allowed for singular unused bytes elements",
			));
		}

		Ok(Self {
			attribute: metabyte_attribute,
			underscore_token: input.parse()?,
		})
	}
}

impl ParseWithContext for ArrayUnused {
	type Context<'a> = (
		usize,
		ParsedAttributes,
		(IdentMap<'a>, IdentMap<'a>),
		DefinitionType,
	);

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self>
	where
		Self: Sized,
	{
		let (
			unused_index,
			ParsedAttributes {
				attributes,
				metabyte_attribute,
				context_attribute,
				sequence_attribute,
			},
			maps,
			definition_type,
		) = context;

		if let Some(attribute) = metabyte_attribute {
			return Err(Error::new(
				attribute.span(),
				"metabyte attributes are not allowed for array-type unused bytes elements",
			));
		}

		if let Some(attribute) = context_attribute {
			return Err(Error::new(
				attribute.span(),
				"context attributes are not allowed for array-type unused bytes elements",
			));
		}

		if let Some(attribute) = sequence_attribute {
			return Err(Error::new(
				attribute.span(),
				"sequence attributes are not allowed for array-type unused bytes elements",
			));
		}

		let content;

		Ok(Self {
			id: UnusedId::new(unused_index),

			attributes,

			bracket_token: bracketed!(content in input),
			underscore_token: content.parse()?,
			semicolon_token: content.parse()?,
			content: content.parse_with((maps, definition_type))?,
		})
	}
}

impl ParseWithContext for UnusedContent {
	type Context<'a> = ((IdentMap<'a>, IdentMap<'a>), DefinitionType);

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self>
	where
		Self: Sized,
	{
		let ((let_map, field_map), definition_type) = context;

		Ok(if input.peek(Token![..]) {
			Self::Infer(input.parse()?)
		} else {
			Self::Source(Box::new(
				input.parse_with(((let_map, Some(field_map)), definition_type))?,
			))
		})
	}
}

impl ParseWithContext for Let<'_> {
	type Context<'a> = (ParsedAttributes, IdentMapMut<'a>, DefinitionType);

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self>
	where
		Self: Sized,
	{
		let (
			ParsedAttributes {
				attributes,
				metabyte_attribute,
				context_attribute,
				sequence_attribute,
			},
			let_map,
			definition_type,
		) = context;

		if let Some(attribute) = sequence_attribute {
			return Err(Error::new(
				attribute.span(),
				"sequence attributes are not allowed for let elements",
			));
		}

		let let_token = input.parse()?;

		let ident: Ident = input.parse()?;
		let colon_token = input.parse()?;
		let r#type: Type = input.parse()?;

		let equals_token = input.parse()?;

		let source = input.parse_with(((&*let_map, None), definition_type))?;

		let id = LetId::new(&ident);
		let_map.insert(id.to_string(), r#type.to_owned());

		Ok(Self {
			id,

			attributes,
			metabyte_attribute,
			context_attribute,

			let_token,

			ident,
			colon_token,
			r#type,

			equals_token,

			source,
		})
	}
}

impl ParseWithContext for Field<'_> {
	type Context<'a> = (usize, ElementType, ParsedAttributes, IdentMapMut<'a>);

	fn parse_with(input: ParseStream, context: Self::Context<'_>) -> Result<Self>
	where
		Self: Sized,
	{
		let (
			index,
			element_type,
			ParsedAttributes {
				attributes,
				metabyte_attribute,
				context_attribute,
				sequence_attribute,
			},
			map,
		) = context;

		let visibility = input.parse()?;
		let ident: Option<(Ident, _)> = match element_type {
			ElementType::Named => Some((input.parse()?, input.parse()?)),
			ElementType::Unnamed => None,
		};
		let r#type: Type = input.parse()?;

		let id = match &ident {
			Some((ident, _)) => FieldId::new_ident(&ident),
			None => FieldId::new_index(index),
		};
		map.insert(id.to_string(), r#type.to_owned());

		Ok(Self {
			id,

			attributes,
			metabyte_attribute,
			context_attribute,
			sequence_attribute,

			visibility,
			ident,
			r#type,
		})
	}
}
