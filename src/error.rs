/// List of virtual machine errors.
#[derive(Debug, Fail, Eq, PartialEq)]
pub enum Error {
	/// No such integer register.
	#[fail(display = "There are only {} integer register but you try to access register #{}.", _0, _1)]
	NoSuchIntegerRegister(usize, usize),

	/// No such floating register.
	#[fail(display = "There are only {} floating register but you try to access register #{}.", _0, _1)]
	NoSuchFloatingRegister(usize, usize),

	/// Failed to define a function. Check if Opcode::FunctionStart is used correctly.
	#[fail(display = "Error occur when trying to define a function {}.", _0)]
	BrokenFunctionDefinition(&'static str),

	/// Can't find <function_name> in a HashMap with all functions.
	#[fail(display = "Function {} is required, but not defined in your application.", _0)]
	FunctionIsNotDefined(&'static str),

	/// ReturnStack is exhausted. There are more returns than function calls.
	#[fail(display = "Stack with return addresses is exhausted. Yep, that is bad.")]
	ReturnStackExhausted,

	/// FunctionStart or FunctionEnd opcode is met on execution stage.
	#[fail(display = "This is a service opcode. It should be inaccessible from normal application run. Did you jump out of function scope?")]
	OpcodeMustBeUnreachable,

	/// Jump opcode executed with non-existent label name. Maybe your label is out of function scope.
	#[fail(display = "You are about to jump to {} label, but it doesn't exist in current function scope.", _0)]
	LabelDoesNotExist(&'static str),

	/// Requested variable doesn't exist in the provided scope.
	#[fail(display = "You are about to use {} variable, but it doesn't exist in current function scope.", _0)]
	VariableDoesNotExist(&'static str),

	/// Requested variable is located outside the data segment or has 0 size
	#[fail(display = "You have requested a broken variable. You should never see this error message.")]
	DataSegmentError,

	/// If you see this error - there is a huge architecture bug. This case must be forbidden by design!
	#[fail(display = "You are about to jump to {} label, but it is out of current function scope.", _0)]
	RestrictedJumpOutOfScope(&'static str),

	/// This operation is not supported. For example storing a new value into constant.
	#[fail(display = "Unsupported operation. You are doing something terribly wrong.")]
	UnsupportedOperation,

	/// Register received an operand of unsupported type.
	#[fail(display = "Unsupported operand. Probably you are trying to store int into float or vise versa.")]
	UnsupportedOperand,

	/// Not implemented.
	#[fail(display = "This functionality is not implemented yet. Sorry.")]
	NotImplemented,
}