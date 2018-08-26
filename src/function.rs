use std::collections::HashMap;
use std::ops::Range;

/// Function is a next abstraction after an Opcode.
/// The idea is that Function provides some restrictions
/// and extra safety.
/// For example, jumps are possible only inside Functions.
#[derive(Debug)]
pub struct Function {
	/// Next index after function definition Opcode.
	//opcode_start_index: usize,
	//opcode_end_index: usize,
	pub name: &'static str,
	pub opcodes_range: Range<usize>,
	pub stackframe_size: usize,

	// A HashMap with labels that are defined inside
	pub labels: HashMap<&'static str, usize>,
	// Argument list
	// Return list
}

impl Function {
	pub fn is_opcode_in_range(&self, opcode_offset: usize) -> bool {
		// strict '>' because END in range should point to return with no exceptions.
		// TODO: Use #![feature(range_contains)] when it is stable
		self.opcodes_range.start <= opcode_offset && self.opcodes_range.end > opcode_offset
	}
}