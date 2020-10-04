use std::iter::FusedIterator;

pub fn variant<S, T>(list: &[S]) -> Variant<T>
where
	S: AsRef<[T]>,
{
	let inputs = list.iter().map(|s| s.as_ref()).collect();
	Variant::new(inputs)
}

/// An iterator that return a combination of element from the given arrays.
///
/// An empty input will return a single empty `Vec`.
/// Which mean the output is always non-zero.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct Variant<'a, T> {
	indices: Vec<usize>,
	inputs: Vec<&'a [T]>,
	first: bool,
}

impl<'a, T> Variant<'a, T> {
	pub fn new(inputs: Vec<&'a [T]>) -> Self {
		let size = inputs.len();
		let indices = vec![0; size];

		let first = true;

		Self {
			indices,
			inputs,
			first,
		}
	}

	fn last_index(&self) -> Option<usize> {
		self.inputs.len().checked_sub(1)
	}

	fn input_len(&self, index: usize) -> Option<usize> {
		self.inputs.get(index).map(|v| v.len())
	}

	fn get(&self, index: usize, at: usize) -> Option<&'a T> {
		self.inputs.get(index).and_then(|v| v.get(at))
	}

	fn increment(&mut self, index: usize) -> Option<()> {
		let len = self.input_len(index)?;
		let indice = self.indices.get_mut(index)?;
		*indice += 1;

		match (index, indice) {
			(0, i) if *i >= len => None,
			(_, i) if *i >= len => {
				*i = 0;
				self.increment(index - 1)
			}
			_ => Some(()),
		}
	}

	fn assemble(&self) -> Option<Vec<&'a T>> {
		self.indices
			.iter()
			.enumerate()
			.map(|(n, i)| self.get(n, *i))
			.collect()
	}
}

impl<'a, T> Variant<'a, T> {
	pub fn total_size(&self) -> usize {
		self.inputs.iter().map(|v| v.as_ref().len()).product()
	}
}

impl<'a, T> Iterator for Variant<'a, T> {
	type Item = Vec<&'a T>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.first {
			self.first = false;
		} else {
			let n = self.last_index()?;
			self.increment(n)?;
		}

		self.assemble()
	}
}

impl<'a, T> FusedIterator for Variant<'a, T> {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn produce_variants() {
		let list = vec![vec!['a', 'b', 'c'], vec!['0', '1'], vec!['♥']];

		let variant = variant(&list);
		let result: Vec<Vec<_>> = variant.map(|c| c.into_iter().copied().collect()).collect();

		let expect = vec![
			vec!['a', '0', '♥'],
			vec!['a', '1', '♥'],
			vec!['b', '0', '♥'],
			vec!['b', '1', '♥'],
			vec!['c', '0', '♥'],
			vec!['c', '1', '♥'],
		];

		assert_eq!(result, expect)
	}

	#[test]
	fn empty_input() {
		let variant = Variant::<()>::new(vec![]);
		let result = variant.count();
		assert_eq!(result, 1);
	}
}
