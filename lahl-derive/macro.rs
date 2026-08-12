use proc_macro::TokenStream;
use std::borrow::Cow;
use syraise::{Fields, Item, emit::compile_error};

#[proc_macro_derive(FromArgs, attributes(group, fall_through, positional))]
pub fn derive_binary_serializable(input: TokenStream) -> TokenStream {
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

				use std::fmt::Write;

				if !branch_definitions.is_empty() {
					branch_definitions.push(',');
				}

				let constructor_branch = &variant.0.name.name;
				let key = to_kebab_case(constructor_branch);

				let description = extract_doc(&variant.0.attributes);

				let _ = write!(
					&mut branch_definitions,
					"::lahl::Branch {{
					prefix: None,
					key: \"{key}\",
					description: \"{description}\",
					aliases: &[],
					result: &[]
				}}"
				);

				let constructor = Constructor::Variant {
					on: constructor_base.to_owned(),
					variant: constructor_branch.to_owned(),
				};

				let inner = fields(&variant.0.fields, &constructor, &mut outside);

				if variant.0.attributes.iter().find(|attr| attr.path == "fall_through").is_some() {
					seen_fall_through = true;
					let _ = write!(
						&mut branch_logic,
						"_ => {{
						{inner}
					}}"
					);
				} else {
					// TODO variants
					let _ = write!(
						&mut branch_logic,
						"Some(\"{key}\") => {{
						{inner}
					}}"
					);
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
				format!(
					"Some(arg) => Err(::lahl::Error::Unknown(arg.to_owned(), &[{options}])),
				None => Err(::lahl::Error::MissingValue)"
				)
			};

			// TODO generics
			let mut out = format!(
				"impl ::lahl::FromArgs for {constructor_base} {{
				fn information() -> ::lahl::Parameters<'static> {{
					&[
						::lahl::Parameter::Switch {{
							name: \"{item_name}\",
							description: \"{description}\",
							branches: &[{branch_definitions}]
						}}
					]
				}}

				fn from_args_instep(args: &mut ::lahl::ArgumentsIter) -> Result<Self, ::lahl::Error> {{
					match args.next().as_deref() {{
						{branch_logic}
						{rest_case}
					}}
				}}
			}}\n"
			);

			out.push_str(&outside);
			out.parse().unwrap()
		}
		Item::Struct(r#struct) => {
			let constructor_base = r#struct.name.name;
			//::lahl::Parameter::Switch {{
			// 	name: \"{item_name}\",
			// 	description: \"{description}\",
			// 	branches: &[{branch_definitions}]
			// }}

			let constructor = Constructor::Struct(constructor_base.clone());
			let mut outside = String::new();
			let body = fields(&r#struct.fields, &constructor, &mut outside);
			let parameters = "";
			let mut out = format!(
				"impl ::lahl::FromArgs for {constructor_base} {{
				fn information() -> ::lahl::Parameters<'static> {{
					&[{parameters}]
				}}

				fn from_args_instep(args: &mut ::lahl::ArgumentsIter) -> Result<Self, ::lahl::Error> {{
					{body}
				}}
			}}\n"
			);

			out.push_str(&outside);
			if let Ok(ok) = out.parse() {
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

fn fields(fields: &Fields, constructor: &Constructor, outside: &mut String) -> String {
	use std::fmt::Write;

	match fields {
		Fields::Unit => format!("Ok({})", constructor.full()),
		Fields::UnnamedFields(fields) => {
			let mut buf = format!("Ok({}(", constructor.full());
			for (idx, _field) in fields.iter().enumerate() {
				if idx > 0 {
					buf.push(',');
				}
				buf.push_str("::lahl::FromArgs::from_args_instep(args)?");
			}
			buf.push_str("))");
			buf
		}
		Fields::NamedFields(fields) => {
			let outer = partial::build_partial(constructor, fields);
			outside.push_str(&outer);

			let (constructor, name): (&str, &str) = match constructor {
				Constructor::Struct(name) => ("", name),
				Constructor::Variant { on, variant } => (on, variant),
			};
			let mut inner = String::new();
			// TODO cannot use Formable, because of constructor ..
			let _ = writeln!(
				&mut inner,
				"let mut partial = <Partial{constructor}{name} as ::lahl::Partial>::new();"
			);
			for field in fields {
				if field.attributes.iter().any(|attr| attr.path == "positional") {
					let _ = writeln!(
						&mut inner,
						"::lahl::Partial::set(&mut partial, \"--{name}\", args)?;",
						name = field.name
					);
				}
			}
			let _ = writeln!(
				&mut inner,
				"while let Some(key) = args.key() {{ ::lahl::Partial::set(&mut partial, &key, args)?; }}"
			);
			let _ = writeln!(&mut inner, "::lahl::Partial::finish(partial)");
			inner
		}
	}
}

/// ```rust ignore
/// assert_eq!(crate::to_kebab_case("hi"), "hi");
/// assert_eq!(crate::to_kebab_case("A"), "a");
/// assert_eq!(crate::to_kebab_case("ChangeDirectory"), "change-directory");
/// assert_eq!(crate::to_kebab_case("field_a"), "field-a");
/// ```
fn to_kebab_case(on: &str) -> Cow<'_, str> {
	let mut buf = Cow::Borrowed("");
	let mut upto = 0;
	for (idx, _) in on.match_indices(|c: char| c.is_ascii_uppercase() || c == '_') {
		let buf = buf.to_mut();
		buf.push_str(&on[upto..idx]);
		if upto > 0 {
			buf.push('-');
		}
		upto = idx;
		let first = on[idx..].chars().next().unwrap();
		if first == '_' {
			buf.push('-');
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

enum Constructor {
	Struct(String),
	Variant { on: String, variant: String },
}

impl Constructor {
	pub fn full(&self) -> String {
		match self {
			Constructor::Struct(name) => name.to_owned(),
			Constructor::Variant { on, variant } => format!("{on}::{variant}"),
		}
	}

	pub fn whole(&self) -> &str {
		match self {
			Constructor::Struct(name) => name,
			Constructor::Variant { on, variant: _ } => on,
		}
	}
}

mod partial {
	use super::Constructor;
	use super::to_kebab_case;
	use std::fmt::Write;
	use syraise::Field;

	pub fn build_partial(constructor: &Constructor, fields: &[Field<String>]) -> String {
		let full_constructor = constructor.full();
		let (parent, name): (&str, &str) = match constructor {
			Constructor::Struct(name) => ("", name),
			Constructor::Variant { on, variant } => (on, variant),
		};

		let mut def_buf = format!("struct Partial{parent}{name} {{\n");
		let mut match_buf = format!("match key {{\n");
		let mut init_buf = format!("Partial{parent}{name} {{\n");
		let mut finish_buf = format!("Ok({full_constructor} {{\n");

		let mut options = String::new();

		for field in fields {
			let match_name = to_kebab_case(&field.name);

			if field.attributes.iter().find(|attr| attr.path == "group").is_some() {
				let _ = writeln!(&mut def_buf, "  {name}: <{type} as ::lahl::Formable>::Partial,", name=field.name, type=field.r#type.0);
				let _ = writeln!(&mut init_buf, "      {name}: <{type} as ::lahl::Formable>::new(),", name = field.name, type=field.r#type.0);
				let _ = writeln!(
					&mut match_buf,
					"      arg if <{type} as ::lahl::Formable>::options().contains(&arg) => {{ ::lahl::Partial::set(&mut self.{name}, key, args)?; }},",
					name=field.name,
					type = field.r#type.to_string()
				);

				let _ = writeln!(
					&mut finish_buf,
					"      {name}: self.{name}.finish()?,",
					name = field.name
				);
			} else {
				let _ = writeln!(&mut def_buf, "  {name}: Option<{type}>,", name=field.name, type=field.r#type.0);
				let _ = writeln!(&mut init_buf, "      {name}: None,", name = field.name);
				let _ = writeln!(
					&mut match_buf,
					"      \"--{match_name}\" => {{ self.{name} = ::lahl::FromArgs::from_args_instep(args)?; }},",
					name = field.name
				);
				let _ = writeln!(
					&mut finish_buf,
					"      {name}: if let Some(value) = self.{name} {{ value }} else {{ ::lahl::FromArgs::elided()? }},",
					name = field.name
				);
			}

			if !options.is_empty() {
				options.push(',');
			}
			let _ = writeln!(&mut options, "\"--{match_name}\"");
		}

		let _ = writeln!(
			&mut match_buf,
			"      key => {{ return Err(::lahl::Error::Unknown(key.to_owned(), Self::options())) }},"
		);

		let _ = write!(&mut def_buf, "}}");
		let _ = write!(&mut match_buf, "    }}");
		let _ = write!(&mut finish_buf, "    }})");
		let _ = write!(&mut init_buf, "    }}");

		// TODO generics
		let impl_buf = {
			let mut buf = format!("impl ::lahl::Partial for Partial{parent}{name} {{\n");
			let _ = writeln!(&mut buf, "  type Out = {name};", name = constructor.whole());
			let _ =
				writeln!(&mut buf, "  fn options() -> &'static [&'static str] {{ &[{options}] }}");
			let _ = writeln!(&mut buf, "  fn new() -> Self {{ {init_buf} }}");
			let _ = writeln!(
				&mut buf,
				"  fn set(&mut self, key: &str, args: &mut ::lahl::ArgumentsIter) -> Result<(), ::lahl::Error> {{ {match_buf}; Ok(())  }}"
			);
			let _ = writeln!(
				&mut buf,
				"  fn finish(self) -> Result<Self::Out, ::lahl::Error> {{ {finish_buf} }}",
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
