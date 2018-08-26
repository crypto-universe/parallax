use std::ops::Add;
use std::ops::Sub;

use error::Error;

#[derive(Debug, Clone, Copy)]
/// Operand type. 
pub enum OperandType {
	/// General purpose register of given number.
	IntegerRegister(usize),
	#[cfg(float)]
	/// Floating point register.
	FloatingRegister(usize),
	/// Address in the memory to read data from. Data size equals register size.
	Memory(usize),
	/// u64 constant.
	IntegerConstant(i64),
	#[cfg(float)]
	/// f64 constant.
	FloatingConstant(f64),
}

#[derive(Debug, Clone, Copy)]
/// Operand value. Returned by prefetcher and differs by data type (like Either).
pub enum OperandValue {
	IntegerValue(i64),
	#[cfg(float)]
	FloatingValue(f64),
	MemoryAddress(usize),
}

impl OperandValue {
	pub fn unwrap_integer(self) -> Result<i64, Error>{
		let result = match self {
			OperandValue::IntegerValue(val) => Ok(val),
			_ => Err(Error::UnsupportedOperand),
		};
		result
	}

	#[cfg(float)]
	pub fn unwrap_floating(self) -> Result<f64, Error>{
		let result = match self {
			OperandValue::FloatingValue(val) => Ok(val),
			_ => Err(Error::UnsupportedOperand),
		};
		result
	}
}

impl Add for OperandValue {
	type Output = Result<OperandValue, Error>;

	fn add(self, other: OperandValue) -> Self::Output {
		match self {
			OperandValue::IntegerValue(val) => {
				Ok(OperandValue::IntegerValue(val + other.unwrap_integer()?))
			},
			#[cfg(float)]
			OperandValue::FloatingValue(val) => {
				Ok(OperandValue::FloatingValue(val + other.unwrap_floating()?))
			},
			_ => Err(Error::NotImplemented),
		}
	}
}

impl Sub for OperandValue {
	type Output = Result<OperandValue, Error>;

	fn sub(self, other: OperandValue) -> Self::Output {
		match self {
			OperandValue::IntegerValue(val) => {
				Ok(OperandValue::IntegerValue(val - other.unwrap_integer()?))
			},
			#[cfg(float)]
			OperandValue::FloatingValue(val) => {
				Ok(OperandValue::FloatingValue(val - other.unwrap_floating()?))
			},
			_ => Err(Error::NotImplemented),
		}
	}
}
