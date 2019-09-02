use std::collections::HashMap;
use std::time::Instant;
use std::mem::{discriminant}; // discriminant will allow to compare enum variants
use std::ops::Range;

use error::Error;
use operand::{OperandValue, OperandType};
use function::Function;
use opcode::Opcode;

/// A simple virtual machine with a stack.
#[derive(Default, Debug)]
pub struct ParallaxVm {
	integer_register: [i64; 32],
	#[cfg(float)] // Make floating point extension optional
	floating_register: [f64; 32],

	opcode_pointer: usize,
	stack_pointer: usize,

	/// This stack holds return address and a stack frame index
	/// Contains function name, opcode pointer and stack pointer
	return_stack: Vec<(&'static str, usize, usize)>,
}

impl ParallaxVm {
	/// Get value depending on operands
	fn prefetch_operand(&self, operand: OperandType) -> Result<OperandValue, Error> {
		match operand {
			OperandType::IntegerRegister(n) => Ok(OperandValue::IntegerValue(self.get_int_register(n)?)),
			OperandType::IntegerConstant(n) => Ok(OperandValue::IntegerValue(n)),
			OperandType::Memory(_address)   => Err(Error::NotImplemented),
			/*_                               => Err(Error::NotImplemented),*/
		}
	}

	/// Set value depending on operands
	fn store_value(&mut self, operand: OperandType, new_value: OperandValue) -> Result<(), Error> {
		match operand {
			OperandType::IntegerRegister(n)  => Ok(self.set_int_register(n, new_value.unwrap_integer()?))?,
			#[cfg(float)]
			OperandType::FloatingRegister(n) => Ok(self.set_float_register(n, new_value.unwrap_floating()?))?,
			OperandType::IntegerConstant(_n) => Err(Error::UnsupportedOperation),
			#[cfg(float)]
			OperandType::FloatingConstant(n) => Err(Error::UnsupportedOperation),
			OperandType::Memory(_address)    => Err(Error::NotImplemented),
		}
	}

	/// Generic implementation of all kinds of jumps.
	fn jump_generic<'x, F>(&mut self, current_func: &'x Function, label_name: &'static str,
			predicate: F, arg1: OperandType, arg2: OperandType)
			-> Result<(&'x Function), Error> where F: FnOnce(i64, i64) -> bool
	{
		let jmp_dst: usize = *current_func.labels.get(label_name).ok_or(Error::LabelDoesNotExist(label_name))?;
		if current_func.is_opcode_in_range(jmp_dst) {
			let arg_val1: i64 = (self.prefetch_operand(arg1)?).unwrap_integer()?;
			let arg_val2: i64 = (self.prefetch_operand(arg2)?).unwrap_integer()?;
			if predicate(arg_val1, arg_val2) {
				self.opcode_pointer = jmp_dst;
			}
			Ok(current_func)
		} else {
			// Should never ever happen. Pray if you see this message.
			Err(Error::RestrictedJumpOutOfScope(current_func.name))
		}
	}

	/// Performs an "action" on 2 arguments and stores the result into dst.
	fn two_operand_action_generic<F>(&mut self, action: F, dst: OperandType, arg1: OperandType, arg2: OperandType)
			-> Result<(), Error> where F: FnOnce(OperandValue, OperandValue) -> Result<OperandValue, Error> 
	{
		let src_val1 = self.prefetch_operand(arg1)?;
		let src_val2 = self.prefetch_operand(arg2)?;
		// Risky behavior - increment before actual action is done.
		// Advantage - cleaner code. There is no need to store temporary result.
		self.opcode_pointer += 1;
		self.store_value(dst, action(src_val1, src_val2)?)
	}

	/// A single "turn" of a virtual machine, i.e. processing a single operation.
	/// Returns reference to current executing Function and stack depth.
	fn turn<'v>(&mut self, operation: &Opcode, current_func: &'v Function, functions: &'v HashMap<&'static str, Function>)
			-> Result<(&'v Function), Error> {
		match *operation {
			Opcode::FunctionStart(_name) => {Err(Error::OpcodeMustBeUnreachable)},
			Opcode::FunctionEnd          => {Err(Error::OpcodeMustBeUnreachable)},
			Opcode::I08(_) | Opcode::I16(_) | Opcode::I32(_) | Opcode::I64(_)     => {
				//Actually we do nothing. This is a variable definition.
				self.opcode_pointer += 1;
				Ok(current_func)
			},
			Opcode::Call(name) => {
				//println!("call {}", name);
				let next_func: &'v Function = functions.get(name).ok_or(Error::FunctionIsNotDefined(name))?;
				// TODO: Create some recursion monitor that can kill app before stack is exhausted?
				self.return_stack.push((current_func.name, self.opcode_pointer + 1, self.stack_pointer));
				self.opcode_pointer = next_func.opcodes_range.start;
				self.stack_pointer += next_func.stackframe_size;
				Ok(next_func)
			},
			Opcode::Return => {
				//println!("return");
				let ret: (&'static str, usize, usize) = self.return_stack.pop().ok_or(Error::ReturnStackExhausted)?;
				let previous_func: &'v Function = functions.get(ret.0).ok_or(Error::FunctionIsNotDefined(ret.0))?;
				// TODO: Check if the address points out of current function scope (impossible case, but still).
				// Recursion should be allowed.
				self.opcode_pointer = ret.1;
				// TODO: Should I subtract current_function.stackframe_size and then verify correctness?
				self.stack_pointer = ret.2;
				Ok(previous_func)
			},
			//=================================================================================================
			Opcode::Label(_name) => {
				//println("label {}", _name);
				//Actually we do nothing. Label is a service opcode, needed on Function init.
				self.opcode_pointer += 1;
				Ok(current_func)
			},
			Opcode::Jump(name) => {
				//println("jump to {} label", name);
				self.jump_generic(current_func, name, |_, _| true,
						OperandType::IntegerConstant(0), OperandType::IntegerConstant(0))
			},
			Opcode::JumpZero(name, arg1) => {
				//println("jump_zero to {} label", name);
				self.jump_generic(current_func, name, |x, _| x == 0, arg1, OperandType::IntegerConstant(0))
			},
			Opcode::JumpNotZero(name, arg1) => {
				//println("jump_not_zero to {} label", name);
				self.jump_generic(current_func, name, |x, _| x != 0, arg1, OperandType::IntegerConstant(0))
			},
			Opcode::JumpBelow(name, arg1, arg2) => {
				//println("jump_below to {} label", name);
				self.jump_generic(current_func, name, |x, y| x < y, arg1, arg2)
			},
			Opcode::JumpBelowEqual(name, arg1, arg2) => {
				//println("jump_below_eq to {} label", name);
				self.jump_generic(current_func, name, |x, y| x <= y, arg1, arg2)
			},
			Opcode::JumpAbove(name, arg1, arg2) => {
				//println("jump_above to {} label", name);
				self.jump_generic(current_func, name, |x, y| x > y, arg1, arg2)
			},
			Opcode::JumpAboveEqual(name, arg1, arg2) => {
				//println("jump_above_eq to {} label", name);
				self.jump_generic(current_func, name, |x, y| x >= y, arg1, arg2)
			},
			Opcode::JumpEqual(name, arg1, arg2) => {
				//println("jump_equal to {} label", name);
				self.jump_generic(current_func, name, |x, y| x == y, arg1, arg2)
			},
			Opcode::JumpNotEqual(name, arg1, arg2) => {
				//println("jump_not_equal to {} label", name);
				self.jump_generic(current_func, name, |x, y| x != y, arg1, arg2)
			},
			//=================================================================================================
			Opcode::Move(dst, src) => {
				//println!("move");
				let src_val = self.prefetch_operand(src)?;
				self.store_value(dst, src_val)?;
				self.opcode_pointer += 1;
				Ok(current_func)
			},
			Opcode::Add(dst, src1, src2) => {
				//println!("add");
				self.two_operand_action_generic(|x, y| x + y, dst, src1, src2)?;
				Ok(current_func)
			},
			Opcode::Sub(dst, src1, src2) => {
				//println!("subtract");
				self.two_operand_action_generic(|x, y| x - y, dst, src1, src2)?;
				Ok(current_func)
			},
		}
	}

	/// Executes given operations on the machine.
	/// Returns number of seconds spent on execution.
	pub fn run(&mut self, program: &[Opcode]) -> Result<u64, Error> {
		let start_time = Instant::now();

		// Functions Map is a result of parsing and it is external to VM
		// (came as list of opcodes), so I don't put them in VM's structure.
		let mut functions: HashMap<&'static str, Function> = HashMap::new();

		// Collect all available functions
		for (i, &op) in program.iter().enumerate() {
			if let Opcode::FunctionStart(name) = op {
				// TODO: change to Error
				assert!(!functions.contains_key(name), "Function {} was already defined before!", name);
				let current_func: Function = self.define_function(name, i, &program[i..])?;
				functions.insert(name, current_func);
			}
		}

		// Start from entry point - "main" function
		let main_func_name: &'static str = "main";
		let mut current_function: &Function = functions
				.get(main_func_name)
				.ok_or(Error::FunctionIsNotDefined(main_func_name))?;

		{
			// Init stack (empty right now), first opcode to start with and return address.
			// TODO: is it OK to write last main's opcode address as a return address?
			self.stack_pointer = 0;
			self.opcode_pointer = current_function.opcodes_range.start;
			self.return_stack.push((main_func_name, current_function.opcodes_range.end, self.stack_pointer));
		}

		while !self.return_stack.is_empty() {
			// Redundant check that should never fail.
			if current_function.is_opcode_in_range(self.opcode_pointer) {
				let current_opcode: &Opcode = &program[self.opcode_pointer];
				current_function = self.turn(current_opcode, current_function, &functions)?;
			} else {
				panic!("All safety measures failed. Running opcode is out of current function. Aborting...");
			}
		}

		// TODO: change to u128 and milliseconds when it becomes stable.
		let elapsed = start_time.elapsed().as_secs();
		Ok(elapsed)
	}

	/// Define a new function and store it in VM for future use.
	/// index - index of FunctionStart opcode in a whole program
	/// program - SLICE of program starting from index!
	pub fn define_function(&self, fname: &'static str, index: usize, program: &[Opcode]) -> Result<Function, Error> {
		let mut function_result: Function = Function{
			name: fname,
			opcodes_range: Range{start: (index + 1), end: 0},
			stackframe_size: 10,
			labels: HashMap::new(),
			variables: HashMap::new(),
			data_segment: Vec::with_capacity(0),
		};
		if let Opcode::FunctionStart(_name) = program[0] {
			let func_end_disc = discriminant(&Opcode::FunctionEnd);
			let func_end_index_o: Option<usize> = program.iter().position(|&x| discriminant(&x) == func_end_disc);

			if let Some(func_end_index) = func_end_index_o {
				// Mark where function ends
				function_result.opcodes_range.end = index + func_end_index;

				let mut required_data_size = 0;
				// Collect local variables and offsets of all labels.
				for (i, opcode) in program.iter().enumerate().take(func_end_index).skip(1)  {
					match *opcode {
						Opcode::Label(label_name) => {
							// Label offset = global offset (index) + local offset (i)
							function_result.labels.insert(label_name, index + i);
							continue;
						},
						Opcode::FunctionStart(func_name) => {
							// Did you try to define a nested function?
							return Err(Error::BrokenFunctionDefinition(func_name));
						},
						Opcode::I08(var_name) => {
							function_result.variables.insert(var_name, (required_data_size, OperandType::IntegerConstant(1)));
							required_data_size += 1;
							continue
						},
						Opcode::I16(var_name) => {
							function_result.variables.insert(var_name, (required_data_size, OperandType::IntegerConstant(2)));
							required_data_size += 2;
							continue
						},
						Opcode::I32(var_name) => {
							function_result.variables.insert(var_name, (required_data_size, OperandType::IntegerConstant(4)));
							required_data_size += 4;
							continue
						},
						Opcode::I64(var_name) => {
							function_result.variables.insert(var_name, (required_data_size, OperandType::IntegerConstant(8)));
							required_data_size += 8;
							continue
						},
						_ => {/* Not interesting */},
					};
				}
				function_result.data_segment.reserve_exact(required_data_size);
			} else {
				return Err(Error::BrokenFunctionDefinition(fname));
			}
		} else {
			return Err(Error::BrokenFunctionDefinition(fname));
		}

		Ok(function_result)
	}

	/// Get value from integer_register or returns an error
	fn get_int_register(&self, reg_number: usize) -> Result<i64, Error> {
		let number_of_registers = self.integer_register.len();
		if reg_number < number_of_registers {
			Ok(self.integer_register[reg_number])
		} else {
			Err(Error::NoSuchIntegerRegister(number_of_registers, reg_number))
		}
	}

	/// Set value into integer_register or returns an error
	fn set_int_register(&mut self, reg_number: usize, new_value: i64) -> Result<(), Error> {
		let number_of_registers = self.integer_register.len();
		if reg_number < number_of_registers {
			self.integer_register[reg_number] = new_value;
			Ok(())
		} else {
			Err(Error::NoSuchIntegerRegister(number_of_registers, reg_number))
		}
	}

	#[cfg(float)]
	/// Get value from floating_register or returns an error
	fn get_float_register(&self, reg_number: usize) -> Result<f64, Error> {
		let number_of_registers = self.integer_register.len();
		if reg_number < number_of_registers {
			Ok(self.floating_register[reg_number])
		} else {
			Err(Error::NoSuchFloatingRegister(number_of_registers, reg_number))
		}
	}

	#[cfg(float)]
	/// Set value from floating_register or returns an error
	fn set_float_register(&mut self, reg_number: usize, new_value: f64) -> Result<(), Error> {
		let number_of_registers = self.floating_register.len();
		if reg_number < number_of_registers {
			self.floating_register[reg_number] = new_value;
			Ok(())
		} else {
			Err(Error::NoSuchFloatingRegister(number_of_registers, reg_number))
		}
	}

	#[cfg(test)]
	/// Get a read-only access to VM's registers for test purposes
	pub fn get_integer_registers(&self) -> &[i64] {
		&self.integer_register
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	/// Helper function, that wraps piece of code into main function.
	fn wrap_into_main(piece_of_code: &mut Vec<Opcode>) -> Vec<Opcode> {
		let mut result_app = Vec::with_capacity(piece_of_code.len() + 3);
		result_app.push(Opcode::FunctionStart("main"));
		result_app.append(piece_of_code);
		result_app.push(Opcode::Return);
		result_app.push(Opcode::FunctionEnd);
		result_app
	}

	#[test]
	fn check_move() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Move(OperandType::IntegerRegister(1), OperandType::IntegerConstant(0xE1EE7)),
			Opcode::Move(OperandType::IntegerRegister(8), OperandType::IntegerConstant(-5)),
			Opcode::Move(OperandType::IntegerRegister(5), OperandType::IntegerRegister(8)),
		]);
		let run_result = vm.run(application.as_slice());
		assert!(run_result.is_ok());
		assert_eq!(vm.get_integer_registers(),
			&[0, 925415, 0, 0, 0, -5, 0, 0, -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	}

	#[test]
	fn check_move_fail1() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Move(OperandType::IntegerConstant(1), OperandType::IntegerConstant(0xE1EE7)),
		]);
		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::UnsupportedOperation));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_move_fail2() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Move(OperandType::IntegerRegister(0), OperandType::IntegerConstant(10)),
			Opcode::Move(OperandType::IntegerConstant(1), OperandType::IntegerRegister(0)),
		]);
		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::UnsupportedOperation));
		assert_eq!(vm.get_integer_registers(),
			&[10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	}

	#[test]
	fn check_nested_function_fail() {
		let mut vm = ParallaxVm::default();
		let func_name = "my_pretty_nested_function";
		let application = wrap_into_main(&mut vec![
			Opcode::FunctionStart(func_name),
		]);
		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::BrokenFunctionDefinition(func_name)));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_no_main_function_fail() {
		let mut vm = ParallaxVm::default();
		let application = &mut vec![
			Opcode::FunctionStart("main_alternative"),
			Opcode::Return,
			Opcode::FunctionEnd,
		];
		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::FunctionIsNotDefined("main")));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_no_main_function_end_fail() {
		let mut vm = ParallaxVm::default();
		let application = &mut vec![
			Opcode::FunctionStart("main"),
			Opcode::Return,
		];
		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::BrokenFunctionDefinition("main")));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_no_arbitrary_function_end_fail() {
		let mut vm = ParallaxVm::default();
		let arbitrary_func_name = "second";
		let mut application = wrap_into_main(&mut vec![]);
		application.append(&mut vec![
			Opcode::FunctionStart(arbitrary_func_name),
			Opcode::Return,
		]);
		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::BrokenFunctionDefinition(arbitrary_func_name)));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_add() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Move(OperandType::IntegerRegister(0), OperandType::IntegerConstant(7)),
			Opcode::Move(OperandType::IntegerRegister(1), OperandType::IntegerConstant(-5)),
			Opcode::Add(OperandType::IntegerRegister(2), OperandType::IntegerRegister(1), OperandType::IntegerRegister(0)),
			Opcode::Add(OperandType::IntegerRegister(3), OperandType::IntegerRegister(2), OperandType::IntegerConstant(10)),
			Opcode::Add(OperandType::IntegerRegister(4), OperandType::IntegerConstant(28), OperandType::IntegerRegister(3)),
			Opcode::Add(OperandType::IntegerRegister(5), OperandType::IntegerConstant(30), OperandType::IntegerConstant(4)),
			Opcode::Add(OperandType::IntegerRegister(6), OperandType::IntegerConstant(8), OperandType::IntegerConstant(-16)),
		]);
		let run_result = vm.run(application.as_slice());
		assert!(run_result.is_ok());
		assert_eq!(vm.get_integer_registers(),
			&[7, -5, 2, 12, 40, 34, -8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	}

	#[test]
	fn check_add_fail1() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Add(OperandType::IntegerConstant(28), OperandType::IntegerRegister(4), OperandType::IntegerConstant(3)),
		]);

		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::UnsupportedOperation));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_sub() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Move(OperandType::IntegerRegister(0), OperandType::IntegerConstant(7)),
			Opcode::Move(OperandType::IntegerRegister(1), OperandType::IntegerConstant(-5)),
			Opcode::Sub(OperandType::IntegerRegister(2), OperandType::IntegerRegister(1), OperandType::IntegerRegister(0)),
			Opcode::Sub(OperandType::IntegerRegister(3), OperandType::IntegerRegister(2), OperandType::IntegerConstant(10)),
			Opcode::Sub(OperandType::IntegerRegister(4), OperandType::IntegerConstant(28), OperandType::IntegerRegister(3)),
			Opcode::Sub(OperandType::IntegerRegister(5), OperandType::IntegerConstant(30), OperandType::IntegerConstant(4)),
			Opcode::Sub(OperandType::IntegerRegister(6), OperandType::IntegerConstant(8), OperandType::IntegerConstant(-16)),
		]);
		let run_result = vm.run(application.as_slice());
		assert!(run_result.is_ok());
		assert_eq!(vm.get_integer_registers(),
			&[7, -5, -12, -22, 50, 26, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	}

	#[test]
	fn check_sub_fail1() {
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::Sub(OperandType::IntegerConstant(28), OperandType::IntegerRegister(4), OperandType::IntegerConstant(3)),
		]);

		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::UnsupportedOperation));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn check_no_label_jump_fail() {
		let label_name = "v1";
		let mut vm = ParallaxVm::default();
		let application = wrap_into_main(&mut vec![
			Opcode::JumpNotZero(label_name, OperandType::IntegerRegister(29)),
		]);

		let run_result = vm.run(application.as_slice());
		assert_eq!(run_result, Err(Error::LabelDoesNotExist(label_name)));
		assert_eq!(vm.get_integer_registers(), &[0; 32]);
	}

	#[test]
	fn generic_test1() {
		let mut vm = ParallaxVm::default();
		let application: Vec<Opcode> = vec![
			Opcode::FunctionStart("main"),
			Opcode::Move(OperandType::IntegerRegister(1), OperandType::IntegerConstant(0x25)),
			Opcode::Add(OperandType::IntegerRegister(2), OperandType::IntegerConstant(3), OperandType::IntegerConstant(5)),
			Opcode::Jump("skip_next_opcode"),
			Opcode::Add(OperandType::IntegerRegister(3), OperandType::IntegerRegister(2), OperandType::IntegerConstant(-1)),
			Opcode::Label("skip_next_opcode"),
			Opcode::Move(OperandType::IntegerRegister(10), OperandType::IntegerConstant(17)),
			Opcode::Call("test1"),
			Opcode::Sub(OperandType::IntegerRegister(2), OperandType::IntegerRegister(2), OperandType::IntegerConstant(6)),
			Opcode::Return,
			Opcode::FunctionEnd,
			Opcode::FunctionStart("test1"),
			Opcode::Add(OperandType::IntegerRegister(10), OperandType::IntegerConstant(10), OperandType::IntegerConstant(5)),
			Opcode::JumpEqual("exit", OperandType::IntegerRegister(10), OperandType::IntegerConstant(15)),
			Opcode::Move(OperandType::IntegerRegister(9), OperandType::IntegerConstant(0xFA)),
			Opcode::Label("exit"),
			Opcode::Return,
			Opcode::FunctionEnd,
		];
		let run_result = vm.run(application.as_slice());
		assert!(run_result.is_ok());
		assert_eq!(vm.get_integer_registers(),
			&[0, 37, 2, 0, 0, 0, 0, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
	}
}
