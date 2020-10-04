use crate::template::Template;
use crate::variant::Variant;

#[derive(Debug, Clone)]
pub struct Solver<'a> {
	inner: Variant<'a, String>,
	keys: Vec<String>,
}

impl<'a> Solver<'a> {
	pub fn new(list: Vec<&'a [String]>, keys: Vec<String>) -> Self {
		let inner = Variant::new(list);
		Self { inner, keys }
	}
}

impl<'a> Iterator for Solver<'a> {
	type Item = Template;

	fn next(&mut self) -> Option<Self::Item> {
		let keys = self.keys.iter().cloned();
		let variant = self.inner.next()?.into_iter().map(|s| s.to_string());
		let result = keys.zip(variant).collect();
		Some(result)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::template::Pool;

	fn prepare_pool() -> Pool {
		let mut pool = Pool::default_rule();
		pool.append("color", "red");
		pool.append("color", "blue");
		pool.append("shape", "circle");
		pool.append("shape", "square");

		pool
	}

	#[test]
	fn solve_color() {
		let pool = prepare_pool();

		let keys = &["color".into()];
		let list = pool.intersect(keys).unwrap();
		let solver = Solver::new(list, keys.to_vec());

		let result: Vec<Template> = solver.collect();
		let expect = vec![
			Template::default().insert("color".into(), "red".into()),
			Template::default().insert("color".into(), "blue".into()),
		];

		assert_eq!(result, expect);
	}

	#[test]
	fn solve_shape() {
		let pool = prepare_pool();
		let keys = &["shape".into()];
		let list = pool.intersect(keys).unwrap();
		let solver = Solver::new(list, keys.to_vec());

		let result: Vec<Template> = solver.collect();
		let expect = vec![
			Template::default().insert("shape".into(), "circle".into()),
			Template::default().insert("shape".into(), "square".into()),
		];

		assert_eq!(result, expect);
	}

	#[test]
	fn solve_both() {
		let pool = prepare_pool();
		let keys = &["color".into(), "shape".into()];
		let list = pool.intersect(keys).unwrap();
		let solver = Solver::new(list, keys.to_vec());

		let result: Vec<Template> = solver.collect();
		let expect = vec![
			Template::default()
				.insert("color".into(), "red".into())
				.insert("shape".into(), "circle".into()),
			Template::default()
				.insert("color".into(), "red".into())
				.insert("shape".into(), "square".into()),
			Template::default()
				.insert("color".into(), "blue".into())
				.insert("shape".into(), "circle".into()),
			Template::default()
				.insert("color".into(), "blue".into())
				.insert("shape".into(), "square".into()),
		];

		assert_eq!(result, expect);
	}

	#[test]
	fn empty_input() {
		let keys = vec![];
		let list = vec![];
		let solver = Solver::new(list, keys);

		let result: Vec<_> = solver.collect();
		let expect = vec![Template::default()];

		assert_eq!(result, expect);
	}
}
