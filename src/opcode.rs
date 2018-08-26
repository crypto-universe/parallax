use operand::OperandType;

#[derive(Debug, Clone, Copy)]
/// Operation code.
pub enum Opcode {
//======================== FUNCTION-RELATED ===================================
	/// Special marker that defines function start
	FunctionStart(&'static str),
	/// Function end.
	FunctionEnd,
	/// Call function by it's name
	Call(&'static str),
	/// Return from function to previous execution place
	Return,
//============================== JUMPS ========================================
	/// Label is also an opcode, but dummy. Used to be jumped to.
	Label(&'static str),
	/// Unconditional jump
	Jump(&'static str),
	/// Jump if operand equals zero
	JumpZero(&'static str, OperandType),
	/// Jump if operand is NOT zero
	JumpNotZero(&'static str, OperandType),
	/// Jump if arg1 < arg2
	JumpBelow(&'static str, OperandType, OperandType),
	/// Jump if arg1 <= arg2
	JumpBelowEqual(&'static str, OperandType, OperandType),
	/// Jump if arg1 > arg2
	JumpAbove(&'static str, OperandType, OperandType),
	/// Jump if arg1 >= arg2
	JumpAboveEqual(&'static str, OperandType, OperandType),
	/// Jump if arg1 == arg2
	JumpEqual(&'static str, OperandType, OperandType),
	/// Jump if arg1 != arg2
	JumpNotEqual(&'static str, OperandType, OperandType),
//============================== MOVES ========================================
	/// Move values into registers (or memory). Destination can't be a constant
	Move(OperandType, OperandType),
//============================== MATH =========================================
	/// Stores in to destination (first argument) sum of two arbitrary operands
	Add(OperandType, OperandType, OperandType),
	/// Stores in to destination (first argument) sub of two arbitrary operands
	Sub(OperandType, OperandType, OperandType),
}