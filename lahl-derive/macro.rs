use std::borrow::Cow;
use std::fmt::Write;

use proc_macro::TokenStream;
use syraise::{Fields, Item, Publicity, emit::compile_error};

#[proc_macro_derive(FromArgs, attributes(lahl))]
pub fn derive_from_args(input: TokenStream) -> TokenStream {
	// dbg!(input.to_string());
	let input = syraise::parse_item(input.into()).unwrap();

	match input {
		Item::Enum(r#enum) => {
			let syraise::Enum { name, variants, attributes, .. } = r#enum;

			let description = extract_doc(&attributes);

			let constructor_base = &name.name;
			let item_name = to_kebab_case(constructor_base);

			let mut branch_definitions = String::new();
			let mut branch_logic = String::new();
			let mut outside = String::new();
			let mut options = String::new();

			let mut seen_fall_through = false;

			for variant in &variants {
				if seen_fall_through {
					return compile_error("two `#[fall_through]` variants").into();
				}

				let constructor_branch = &variant.0.name.name;
				let key = to_kebab_case(constructor_branch);

				let description = extract_doc(&variant.0.attributes);

				let constructor = Constructor::Variant {
					publicity: &r#enum.publicity,
					on: constructor_base,
					variant: constructor_branch,
				};

				let FieldsOut { parse_logic, information } =
					FieldsOut::new(&variant.0.fields, &constructor, &mut outside);

				{
					if !branch_definitions.is_empty() {
						let _ = write!(&mut branch_definitions, ",");
					}

					// TODO literal and unknown_or_after_dash_dash
					let aliases_buf = String::new();
					// for alias in aliases {
					// 	if !aliases_buf.is_empty() {
					// 		let _ = write!(&mut aliases_buf, ",");
					// 	}
					// 	let _ = write!(&mut aliases_buf, "\"{alias}\"");
					// }

					let _ = write!(
						&mut branch_definitions,
						"::lahl::Branch {{ key: \"{key}\", aliases: vec![{aliases_buf}], description: \"{description}\", result: {information} }}"
					);
				}

				let flags = Flags::from_attributes(&variant.0.attributes);
				if flags.fall_through {
					seen_fall_through = true;
					let _ = write!(
						&mut branch_logic,
						"_ => {{ if let Some(arg) = arg {{ args.reject(arg); }} {parse_logic} }}"
					);
				} else {
					let mut match_equal = format!("\"{key}\"");
					for alias in &flags.aliases {
						let _ = write!(&mut match_equal, "| \"{alias}\"");
					}
					// TODO variants
					let _ = write!(&mut branch_logic, "Some({match_equal}) => {{ {parse_logic} }}");
					branch_logic.push(',');

					if !options.is_empty() {
						options.push(',');
					}
					let _ = writeln!(&mut options, "\"{key}\"");
				}
			}

			let rest_case = if seen_fall_through {
				String::new()
			} else {
				// TODO MissingValue should return the options? or set current
				format!(
					"Some(arg) => Err(::lahl::BaseError::Unknown(arg.to_owned())),
					None => Err(::lahl::BaseError::MissingValue)"
				)
			};

			// TODO generics
			let mut out = format!(
				"impl ::lahl::FromArgs for {constructor_base} {{
					fn information() -> ::lahl::Parameter<'static> {{
						::lahl::Parameter::Switch {{ name: Some(::lahl::NameDescription {{ name: \"{item_name}\", description: \"{description}\" }}), branches: vec![{branch_definitions}] }}
					}}

					fn from_args_instep(args: &mut ::lahl::Arguments) -> Result<Self, ::lahl::BaseError> {{
						let arg = match args.next() {{
							Some(::lahl::ValueOrHelp::Value(arg)) => Some(arg),
							Some(::lahl::ValueOrHelp::Help) => {{ return Err(::lahl::BaseError::Help); }},
							None => None
						}};
						match arg.as_deref() {{
							{branch_logic}
							{rest_case}
						}}
					}}
				}}"
			);

			out.push_str(&outside);
			out.parse().unwrap()
		}
		Item::Struct(r#struct) => {
			let constructor_base = r#struct.name.name;
			let constructor = Constructor::Struct(&r#struct.publicity, &constructor_base);
			let mut outside = String::new();
			let FieldsOut { information, parse_logic } =
				FieldsOut::new(&r#struct.fields, &constructor, &mut outside);
			let mut out = format!(
				"impl ::lahl::FromArgs for {constructor_base} {{
					fn information() -> ::lahl::Parameter<'static> {{
						{information}
					}}

					fn from_args_instep(args: &mut ::lahl::Arguments) -> Result<Self, ::lahl::BaseError> {{
						{parse_logic}
					}}
				}}"
			);

			out.push_str(&outside);
			if let Ok(ok) = out.parse() {
				// eprintln!("{ok}");
				ok
			} else {
				panic!("bad output {out}");
			}
		}
	}
}

fn extract_doc(attributes: &[syraise::Attribute]) -> String {
	let mut buf = String::new();
	for attribute in attributes {
		if attribute.path == "doc" {
			let (_, value) = &attribute.value.as_ref().unwrap();
			let value = value.to_string();
			if let Some(before) = value.strip_prefix('"')
				&& let Some(after) = before.strip_suffix('"')
			{
				buf.push_str(after.trim());
			} else {
				unreachable!("{value}");
			}
		}
	}
	buf
}

#[derive(Default)]
struct Flags {
	pub literal: bool,
	pub group: bool,
	pub fall_through: bool,
	pub positional: bool,
	pub unknown_or_after_dash_dash: bool,
	pub aliases: Vec<String>,
}

impl Flags {
	pub fn from_attributes(attributes: &[syraise::Attribute]) -> Self {
		let mut flags = Flags::default();
		for attribute in attributes {
			if attribute.path == "lahl" {
				let value = attribute.value.as_ref().unwrap().1.clone();
				let mut tokens = value.into_iter();
				let Some(proc_macro2::TokenTree::Ident(initial)) = tokens.next() else {
					panic!();
				};
				let initial = initial.to_string();
				match initial.as_str() {
					"group" => {
						flags.group = true;
					}
					"literal" => {
						flags.literal = true;
					}
					"positional" => {
						flags.positional = true;
					}
					"fall_through" => {
						flags.fall_through = true;
					}
					"unknown_or_after_dash_dash" => {
						flags.unknown_or_after_dash_dash = true;
					}
					"alias" => {
						let is_equal = matches!(tokens.next(), Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == '=');
						assert!(is_equal);
						if let Some(proc_macro2::TokenTree::Literal(initial)) = tokens.next() {
							let inner = initial.to_string();
							if let Some(after) = inner.strip_prefix('"')
								&& let Some(before) = after.strip_suffix('"')
							{
								flags.aliases.push(before.to_owned());
							} else {
								panic!("bad string literal");
							}
						} else {
							panic!("not string literal");
						}
					}
					"aliases" => {
						let Some(proc_macro2::TokenTree::Group(items)) = tokens.next() else {
							panic!("not group");
						};
						let mut inner = items.stream().into_iter();
						while let Some(item) = inner.next() {
							if let proc_macro2::TokenTree::Literal(initial) = item {
								let inner = initial.to_string();
								if let Some(after) = inner.strip_prefix('"')
									&& let Some(before) = after.strip_suffix('"')
								{
									flags.aliases.push(before.to_owned());
								} else {
									panic!("bad string literal");
								}
							} else {
								panic!("not string literal");
							}
							let is_comma = matches!(inner.next(), Some(proc_macro2::TokenTree::Punct(punct)) if punct.as_char() == ',');
							if !is_comma {
								break;
							}
						}
					}
					unknwn => {
						panic!("{unknwn:?}");
					}
				}
				// while let Some(next) =
			}
		}
		flags
	}
}

struct FieldsOut {
	pub parse_logic: String,
	pub information: String,
}

impl FieldsOut {
	pub fn new(fields: &Fields, constructor: &Constructor, outside: &mut String) -> FieldsOut {
		match fields {
			Fields::Unit => FieldsOut {
				parse_logic: format!("Ok({})", constructor.full()),
				information: "::lahl::Parameter::Unit".to_owned(),
			},
			Fields::Unnamed(fields) => {
				FieldsOut {
					parse_logic: {
						let mut buf = format!("Ok({}(", constructor.full());
						for (idx, _) in fields.iter().enumerate() {
							if idx > 0 {
								buf.push(',');
							}
							buf.push_str("::lahl::FromArgs::from_args_instep(args)?");
						}
						buf.push_str("))");
						buf
					},
					information: {
						let mut buf = String::new();
						for field in fields {
							if !buf.is_empty() {
								buf.push(',');
							}
							let _ = write!(&mut buf, "<{type} as ::lahl::FromArgs>::information()", type=field.r#type.to_string());
						}
						let name = match constructor {
							Constructor::Variant { .. } => "None".to_owned(),
							// TODO description from constructor
							Constructor::Struct(_, name) => format!("Some(::lahl::NameDescription {{ name: \"{name}\", description: \"\" }})"),
						};
						format!(
							"::lahl::Parameter::Group {{
								name: {name},
								unnamed: vec![{buf}],
								named: Vec::new()
							}}"
						)
					},
				}
			}
			Fields::Named(fields) => {
				FieldsOut {
					parse_logic: {
						let outer = partial::build_partial(constructor, fields);
						outside.push_str(&outer);

						let (constructor, name): (&str, &str) = match constructor {
							Constructor::Struct(_, name) => ("", name),
							Constructor::Variant { on, variant, .. } => (on, variant),
						};
						let mut inner = String::new();

						// TODO cannot use Formable, because of constructor ..
						let _ = writeln!(
							&mut inner,
							"let mut partial = <Partial{constructor}{name} as ::lahl::Partial>::new();"
						);

						// Unknown catch logic here
						let mut unknown_or_after_dash_dash_field_name: Option<String> = None;
						for field in fields {
							let flags = Flags::from_attributes(&field.attributes);
							if flags.positional {
								let prefix = if flags.literal { "" } else { "--" };
								let name = to_kebab_case(&field.name);
								let handle_result = "match result {
									Ok(()) => {}
									Err(::lahl::PartialSetError::Unknown(_name)) => { unreachable!(); }
									Err(::lahl::PartialSetError::Inner(base)) => { return Err(base); }
								}";
								let _ = writeln!(
									&mut inner,
									"let result = ::lahl::Partial::set(&mut partial, \"{prefix}{name}\", args);
									{handle_result}",
								);
							} else if flags.unknown_or_after_dash_dash {
								unknown_or_after_dash_dash_field_name = Some(field.name.to_owned());
							}
						}

						// unknown_or_after_dash_dash_field_name
						let extract_key = "let key = match key {
							::lahl::ValueOrHelp::Value(key) => key,
							::lahl::ValueOrHelp::Help => { return Err(::lahl::BaseError::Help); }
						}";
						let consumer = if let Some(field) = unknown_or_after_dash_dash_field_name {
							// `::lahl::collect(&mut partial.{field}, args); break;`
							format!("if let \"--\" = key.as_str() {{
								::lahl::FromArgs::append(&mut partial.{field}, args)?;
								continue;
							}}")
						} else {
							String::new()
						};
						let set_partial = "let result = ::lahl::Partial::set(&mut partial, &key, args);";
						let handle_result = "match result {
							Ok(()) => {}
							Err(::lahl::PartialSetError::Unknown(name)) => { args.reject(name); break; }
							Err(::lahl::PartialSetError::Inner(base)) => { return Err(base); }
						}";
						let _ = writeln!(
							&mut inner,
							"while let Some(key) = args.key() {{
								{extract_key};
								{consumer}
								{set_partial}
								{handle_result}
							}}
							::lahl::Partial::finish(partial)"
						);
						inner
					},
					information: {
						let mut named = String::new();
						let mut unnamed = String::new();
						for field in fields {
							let flags = Flags::from_attributes(&field.attributes);
							if flags.positional {
								if !unnamed.is_empty() {
									unnamed.push(',');
								}
								let _ = write!(&mut unnamed, "<{type} as ::lahl::FromArgs>::information()", type=field.r#type.to_string());
							} else {
								if !named.is_empty() {
									named.push(',');
								}
								let description = extract_doc(&field.attributes);
								let key = to_kebab_case(&field.name);
								// TODO literal and unknown_or_after_dash_dash
								let mut aliases_buf = String::new();
								for alias in flags.aliases {
									if !aliases_buf.is_empty() {
										let _ = write!(&mut aliases_buf, ",");
									}
									let _ = write!(&mut aliases_buf, "\"{alias}\"");
								}
								let _ = write!(&mut named, "::lahl::Branch {{ key: \"{key}\", aliases: vec![{aliases_buf}], description: \"{description}\", result: <{type} as ::lahl::FromArgs>::information() }}", type=field.r#type.to_string());
							}
						}
						let name = match constructor {
							Constructor::Variant { .. } => "None".to_owned(),
							// TODO description from constructor
							Constructor::Struct(_, name) => format!("Some(::lahl::NameDescription {{ name: \"{name}\", description: \"\" }})"),
						};
						format!(
							"::lahl::Parameter::Group {{
								name: {name},
								unnamed: vec![{unnamed}],
								named: vec![{named}]
							}}"
						)
					},
				}
			}
		}
	}
}

/// ```rust ignore
/// assert_eq!(crate::to_kebab_case("hi"), "hi");
/// assert_eq!(crate::to_kebab_case("A"), "a");
/// assert_eq!(crate::to_kebab_case("ChangeDirectory"), "change-directory");
/// assert_eq!(crate::to_kebab_case("field_a"), "field-a");
/// assert_eq!(crate::to_kebab_case("r#in"), "in");
/// ```
fn to_kebab_case(on: &str) -> Cow<'_, str> {
	// Strip Rust identifier stuff
	let on = on.strip_prefix("r#").unwrap_or(on);

	let mut buf = Cow::Borrowed("");
	let mut upto = 0;
	for (idx, _) in on.match_indices(|c: char| c.is_ascii_uppercase() || c == '_') {
		let buf = buf.to_mut();
		buf.push_str(&on[upto..idx]);
		if upto < idx {
			buf.push('-');
		}
		upto = idx;
		let first = on[idx..].chars().next().unwrap();
		if first == '_' {
			if !buf.ends_with('-') {
				buf.push('-');
			}
			upto += 1;
		} else {
			for c in first.to_lowercase() {
				upto += c.len_utf8();
				buf.push(c);
			}
		}
	}
	buf += &on[upto..];
	buf
}

/// TODO from syraise
enum Constructor<'a> {
	Struct(&'a Publicity, &'a str),
	Variant { publicity: &'a Publicity, on: &'a str, variant: &'a str },
}

impl Constructor<'_> {
	pub fn full(&self) -> String {
		match self {
			Constructor::Struct(_, name) => name.to_string(),
			Constructor::Variant { publicity: _, on, variant } => format!("{on}::{variant}"),
		}
	}

	pub fn whole(&self) -> &str {
		match self {
			Constructor::Struct(_, name) => name,
			Constructor::Variant { publicity: _, on, variant: _ } => on,
		}
	}

	// pub fn leaf(&self) -> &str {
	// 	match self {
	// 		Constructor::Struct(_, name) => name,
	// 		Constructor::Variant { publicity: _, on: _, variant } => variant,
	// 	}
	// }

	pub fn publicity(&self) -> String {
		match self {
			Constructor::Struct(publicity, _) | Constructor::Variant { publicity, .. } => {
				Publicity::to_string(*publicity)
			}
		}
	}
}

mod partial {
	use super::Constructor;
	use super::Flags;
	use super::to_kebab_case;
	use std::fmt::Write;
	use syraise::Field;

	pub fn build_partial(constructor: &Constructor, fields: &[Field<String>]) -> String {
		let full_constructor = constructor.full();
		let (parent, name): (&str, &str) = match constructor {
			Constructor::Struct(_, name) => ("", name),
			Constructor::Variant { on, variant, .. } => (on, variant),
		};

		let publicity = constructor.publicity();
		let mut def_buf = format!("{publicity} struct Partial{parent}{name} {{\n");
		let mut match_buf = format!("match key {{\n");
		let mut init_buf = format!("Partial{parent}{name} {{\n");
		let mut finish_buf = format!("Ok({full_constructor} {{\n");

		let mut options = String::new();
		let mut seen_unknown_or_after_dash_dash: bool = false;

		for field in fields {
			let match_name = to_kebab_case(&field.name);
			let flags = Flags::from_attributes(&field.attributes);
			// nested here
			if flags.group {
				let _ = writeln!(&mut def_buf, "{name}: <{type} as ::lahl::Formable>::Partial,", name = field.name, type = field.r#type.0);
				let _ = writeln!(&mut init_buf, "{name}: <{type} as ::lahl::Formable>::new(),", name = field.name, type = field.r#type.0);
				let _ = writeln!(
					&mut match_buf,
					"arg if <{type} as ::lahl::Formable>::will_eat(&arg) => {{ ::lahl::Partial::set(&mut self.{name}, key, args) }},",
					name=field.name,
					type = field.r#type.to_string()
				);

				let _ =
					writeln!(&mut finish_buf, "{name}: self.{name}.finish()?,", name = field.name);
			} else {
				let _ = writeln!(&mut def_buf, "{name}: Option<{type}>,", name=field.name, type=field.r#type.0);
				let _ = writeln!(&mut init_buf, "{name}: None,", name = field.name);
				if flags.unknown_or_after_dash_dash {
					seen_unknown_or_after_dash_dash = true;
					// Could be append always...
					// if !::std::ptr::eq(::lahl::EMPTY, key) {{ args.reject(key.to_owned()); }}
					let _ = writeln!(
						&mut match_buf,
						"key => {{
							args.reject(key.to_owned());
							if let Some(ref mut value) = self.{name} {{ ::lahl::FromArgs::append(value, args)?; Ok(()) }}
							else {{ self.{name} = Some(::lahl::FromArgs::from_args_instep(args)?); Ok(()) }}
						}},",
						name = field.name
					);
				} else {
					let prefix = if flags.literal { "" } else { "--" };
					let mut match_equal = format!("\"{prefix}{match_name}\"");
					for alias in &flags.aliases {
						let _ = write!(&mut match_equal, "| \"{alias}\"");
					}
					let _ = writeln!(
						&mut match_buf,
						"{match_equal} => {{
							if let Some(ref mut value) = self.{name} {{
								::lahl::FromArgs::append(value, args)?;
							}} else {{
								self.{name} = Some(::lahl::FromArgs::from_args_instep(args)?);
							}}
							Ok(())
						}},",
						name = field.name
					);
					let _ = write!(&mut options, "\"{prefix}{match_name}\",");
					for alias in flags.aliases {
						let _ = write!(&mut match_equal, "\"{alias}\",");
					}
				}
				let _ = writeln!(
					&mut finish_buf,
					"{name}: if let Some(value) = self.{name} {{ value }} else {{ ::lahl::FromArgs::elided()? }},",
					name = field.name
				);
			}
		}

		if !seen_unknown_or_after_dash_dash {
			// hmm
			let _ = writeln!(
				&mut match_buf,
				"key => {{ return Err(::lahl::PartialSetError::Unknown(key.to_owned())) }},"
			);
		}

		let _ = write!(&mut def_buf, "}}");
		let _ = write!(&mut match_buf, "    }}");
		let _ = write!(&mut finish_buf, "    }})");
		let _ = write!(&mut init_buf, "    }}");

		// TODO generics
		let impl_buf = {
			let mut buf = format!("impl ::lahl::Partial for Partial{parent}{name} {{\n");
			let _ = writeln!(&mut buf, "  type Out = {name};", name = constructor.whole());
			// TODO matches?
			let _ = writeln!(
				&mut buf,
				"  fn will_eat(on: &str) -> bool {{ [{options}].contains(&on) }}"
			);
			let _ = writeln!(&mut buf, "  fn new() -> Self {{ {init_buf} }}");
			let _ = writeln!(
				&mut buf,
				"  fn set(&mut self, key: &str, args: &mut ::lahl::Arguments) -> Result<(), ::lahl::PartialSetError> {{ {match_buf} }}"
			);
			let _ = writeln!(
				&mut buf,
				"  fn finish(self) -> Result<Self::Out, ::lahl::BaseError> {{ {finish_buf} }}",
			);
			buf.push('}');
			buf
		};

		let formable_buf = if let Constructor::Variant { .. } = constructor {
			String::new()
		} else {
			format!(
				"impl ::lahl::Formable for {parent}{name} {{ type Partial = Partial{parent}{name}; }}"
			)
		};
		format!("{def_buf} {impl_buf} {formable_buf}")
	}
}
