use std::collections::HashMap;
use std::ops::Range;

use error::Error;
use operand::{OperandType, OperandValue};

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

	/// variables is a map: name -> (offset, Type(size))
	pub variables: HashMap<&'static str, (usize, OperandType)>,

	// TODO: make it global?
	pub data_segment: Vec<u8>,
	// Argument list
	// Return list
}

impl Function {
	pub fn is_opcode_in_range(&self, opcode_offset: usize) -> bool {
		// strict '>' because END in range should point to return with no exceptions.
		// TODO: Use #![feature(range_contains)] when it is stable
		self.opcodes_range.start <= opcode_offset && self.opcodes_range.end > opcode_offset
	}

	pub fn get_var_value(&self, variable_name: &'static str) -> Result<OperandValue, Error> {
		let (offset, type_size): &(usize, OperandType) = self.variables.get(variable_name).ok_or(Error::VariableDoesNotExist(variable_name))?;
		let size = match type_size {
			OperandType::IntegerConstant(size) => *size as usize,
			#[cfg(float)]
			OperandType::FloatingConstant(size) => *size as usize,
			// TODO: support pointers?
			_ => 0,
		};
		if (size == 0) || (offset + size > self.data_segment.len()) {
			return Err(Error::DataSegmentError);
		}

		//let pointer_to_variable = self.data_segment.as_slice()[*offset];
		let mut result: i64 = 0;
		for i in 0..size {
			result = (result << 8) + i64::from(self.data_segment[offset + i]);
		}

		Ok(OperandValue::IntegerValue(result))
	}

	pub fn set_var_value(&self, variable_name: &'static str, value: OperandValue) -> Result<(), Error> {
		Ok(())
	}
}