use crate::{Branch, NameDescription, Parameter};

const SPACES: &str = "                   ";

// TODO use `Write` trait
pub fn print_information(parameter: Parameter<'_>) {
	// dbg!(&parameter);
	write_parameter(parameter, 0)
}

fn write_parameter(parameter: Parameter, depth: usize) {
	let indent_width = depth.saturating_sub(1) * 2;
	match parameter {
		Parameter::Group { name, unnamed, named } => {
			let mut depth = depth;
			if let Some(NameDescription { name, description }) = name {
				let indent = &SPACES[..indent_width];
				eprintln!("{indent}{name} \u{1b}[2m{description}\u{1b}[0m");
				eprintln!();
				depth = depth + 1;
			}
			// let has_unnamed = !unnamed.is_empty();
			for unnamed in unnamed {
				let indent = &SPACES[..indent_width];
				eprint!("{indent}");
				write_parameter(unnamed, depth);
				eprintln!();
			}
			// if has_unnamed {
			// 	eprintln!();
			// }
			let max_key = named.iter().map(|item| item.key.len()).max().unwrap_or_default();
			for named in named {
				let name = named.key;
				let description = named.description;
				let indent = &SPACES[..indent_width];
				let spacing = &SPACES[..max_key - name.len()];
				eprint!(
					"{indent}\u{1b}[33m--{name}\u{1b}[0m{spacing} \u{1b}[2m{description}\u{1b}[0m "
				);
				write_parameter(named.result, depth + 1);
				eprintln!();
			}
		}
		Parameter::Switch { name, branches } => {
			let indent = &SPACES[..indent_width];
			if let Some(NameDescription { name, description }) = name {
				eprintln!("{indent}\u{1b}[31m{name}\u{1b}[0m \u{1b}[2m{description}\u{1b}[0m\n ");
			}
			let depth = depth + 1;
			let max_key = branches.iter().map(|item| item.key.len()).max().unwrap_or_default();
			for branch in branches {
				let Branch { key, aliases: _, description, result } = branch;
				let indent = &SPACES[..indent_width + 2];
				let spacing = &SPACES[..max_key - key.len()];
				eprintln!(
					"{indent}\u{1b}[34m{key}\u{1b}[0m{spacing} \u{1b}[2m{description}\u{1b}[0m "
				);
				write_parameter(result, depth + 2);
				eprintln!();
			}
			// if depth > 1 {
			// 	eprintln!();
			// }
		}
		Parameter::Leaf { optional, name } => {
			if let Some(NameDescription { name, description }) = name {
				eprint!("\u{1b}[35m{name}\u{1b}[0m ");
				if !description.is_empty() {
					eprint!("\u{1b}[2m{description}\u{1b}[0m ");
				}
			} else {
				// eprint!("arg n ??");
			}
			if optional {
				eprint!("(optional)");
			}
			if depth == 0 {
				eprintln!();
			}
		}
		Parameter::Multiple => {}
		Parameter::Unit => {}
	}
}
