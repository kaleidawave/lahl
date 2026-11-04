#![allow(unused)]

fn main() {
	use lahl::FromArgs;

	let value = basic::NamedWithOptionalFields::from_env_args();
	match value {
		Ok(value) => {
			dbg!(value);
		}
		Err(error) => {
			// dbg!(&error);
			error.print();
		}
	}
}

mod basic {
	/// This is named with additional fields
	#[derive(lahl::FromArgs, PartialEq, Debug)]
	pub struct NamedWithOptionalFields {
		optional_value: Option<String>,
		optional_flag: bool,
	}
}
