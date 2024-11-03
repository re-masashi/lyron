
#[derive(Debug)]
enum OpCode {
	Return,
}

#[derive(Debug)]
struct Chunk {
	code: Vec<OpCode>
}