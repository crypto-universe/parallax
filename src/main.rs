#[macro_use]
extern crate failure;

mod error;
mod operand;
mod opcode;
mod function;
mod parallax_vm;

pub use error::Error;
pub use opcode::Opcode;
pub use operand::OperandType;
pub use parallax_vm::ParallaxVm;

fn main() -> Result<(), Error> {
	let mut vm = ParallaxVm::default();
	let application: Vec<Opcode> = vec![
/*01*/		Opcode::FunctionStart("main"),
/*02*/		Opcode::Move(OperandType::IntegerRegister(1), OperandType::IntegerConstant(0xE1EE7)),
/*03*/		Opcode::Add(OperandType::IntegerRegister(2), OperandType::IntegerConstant(3), OperandType::IntegerConstant(5)),
/*04*/		Opcode::Jump("skip_next_opcode"),
/*05*/		Opcode::Add(OperandType::IntegerRegister(3), OperandType::IntegerRegister(2), OperandType::IntegerConstant(-1)),
/*06*/		Opcode::Label("skip_next_opcode"),
/*07*/		Opcode::Call("test1"),
/*08*/		Opcode::Sub(OperandType::IntegerRegister(2), OperandType::IntegerRegister(2), OperandType::IntegerConstant(6)),
/*09*/		Opcode::Return,
/*10*/		Opcode::FunctionEnd,
/*11*/		Opcode::FunctionStart("test1"),
			Opcode::I32("first_var", 56),
/*12*/		Opcode::Add(OperandType::IntegerRegister(10), OperandType::IntegerConstant(31), OperandType::IntegerConstant(5)),
/*13*/		Opcode::Return,
/*14*/		Opcode::FunctionEnd,
	];
	let res = ParallaxVm::run(&mut vm, application.as_slice())?;
	println!("{:?}", vm);
	println!("Execution time: {} seconds.", res);
	Ok(())
}