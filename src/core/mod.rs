mod filter;
mod link;
mod walker;

pub use filter::*;
pub use link::*;
pub use walker::*;

pub mod special {
	use megumax_template::Resource;

	pub const NTH_TEMPLATE: &str = "[nth]";

	pub fn nth_template((n, mut res): (usize, Resource)) -> Resource {
		res.template.set(NTH_TEMPLATE.to_owned(), n.to_string());
		res
	}
}
