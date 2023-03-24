use std::io::Cursor;
use byteorder::{ReadBytesExt, LittleEndian};

mod stdlib;

use crate::compiler::Instruction;


pub fn run(program: &Vec<u8>) -> Result<(), ()> {    
    let mut stack = Vec::<i64>::new();
    let mut memory = Vec::<u8>::new();

    let mut binary = Cursor::new(program);
    let code_offset = binary.read_u64::<LittleEndian>().unwrap();
    binary.set_position(code_offset);

    loop {
        let op = binary.read_u8().unwrap();
        match op {
            x if x == Instruction::Push as u8 => stack.push(binary.read_i64::<LittleEndian>().unwrap()),
            x if x == Instruction::Dup as u8 => stack.push(stack[stack.len() - 1]),
            x if x == Instruction::Over as u8 => stack.push(stack[stack.len() - 2]),
            x if x == Instruction::Add as u8 => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a + b);
            },
            x if x == Instruction::Equal as u8 => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push((a == b) as i64);
            },
            x if x == Instruction::If as u8 => {
                let jump = binary.read_u64::<LittleEndian>().unwrap();
                if stack.pop().unwrap() == 0 {
                    binary.set_position(jump);
                }
            },
            x if x == Instruction::Jump as u8 => {
                let jump = binary.read_u64::<LittleEndian>().unwrap();
                binary.set_position(jump);
            },
            x if x == Instruction::Call as u8 => {
                match binary.read_u64::<LittleEndian>().unwrap() {
                    0x00 => stdlib::putd(stack.pop().unwrap()),
                    0x01 => {
                        let address = stdlib::alloc(&mut memory, stack.pop().unwrap() as usize);
                        stack.push(address as i64);
                    },
                    0x02 => {
                        let address = stack.pop().unwrap() as usize;
                        let size = stack.pop().unwrap() as usize;
                        stdlib::write(&memory[address..address+size]);
                    },
                    _ => todo!(),
                }
            },
            x if x == Instruction::Poke as u8 => {
                let address = stack.pop().unwrap() as usize;
                let value = stack.pop().unwrap() as u8;
                memory[address] = value;
            },
            x if x == Instruction::Exit as u8 => return Ok(()),
            _ => todo!("unimplemeted opcode {:#04X} at address `{}`", op, binary.position() - 1),
        }
    }
}