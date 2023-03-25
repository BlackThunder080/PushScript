use std::{io::{Cursor, Write}, collections::{HashMap, HashSet}};
use byteorder::{WriteBytesExt, LittleEndian};

#[derive(Debug)]
pub enum OP {
    Push(i64),
    PushStr(String),
    Dup,
    Over,
    Add,
    Equal,
    If,
    Else,
    End,
    Call(String),
    Poke,
}

#[repr(u8)]
pub enum Instruction {
    Push = 0x01,
    Dup,
    Over,
    Add,
    Equal,
    If,
    Jump,
    Call,
    Poke,
    Exit = 0xFF,
}

pub fn compile(program: &str) -> Vec<u8> {   
    let program = lex(program);
    let mut binary = Cursor::new(Vec::<u8>::new());
    
    let mut cross_ref_stack = Vec::<(OP, u64)>::new();
    
    let mut data = Vec::<u8>::new();

    let mut functions = HashMap::<&str, u64>::new();
    functions.insert("putd", 0);
    functions.insert("alloc", 1);
    functions.insert("write",2);

    // =====================[ Offset Table ]=======================
    /* Code Offset */ binary.write_u64::<LittleEndian>(16).unwrap();    
    /* Data Offset */ binary.write_u64::<LittleEndian>(0).unwrap();

    // ====================[ Code Section ]========================
    for op in program {
        match op {
            OP::Push(n) => {
                binary.write_u8(Instruction::Push as u8).unwrap();
                binary.write_i64::<LittleEndian>(n).unwrap();
            },
            OP::PushStr(string) => {
                binary.write_u8(Instruction::Push as u8).unwrap();
                binary.write_i64::<LittleEndian>(string.as_bytes().len() as i64).unwrap();
                binary.write_u8(Instruction::Push as u8).unwrap();
                binary.write_i64::<LittleEndian>(data.len() as i64).unwrap();
                data.append(&mut Vec::from(string.as_bytes()));
            }
            OP::Dup => binary.write_u8(Instruction::Dup as u8).unwrap(),
            OP::Over => binary.write_u8(Instruction::Over as u8).unwrap(),
            OP::Add => binary.write_u8(Instruction::Add as u8).unwrap(),
            OP::Equal => binary.write_u8(Instruction::Equal as u8).unwrap(),
            OP::If => {
                binary.write_u8(Instruction::If as u8).unwrap();
                cross_ref_stack.push((op, binary.position()));
                binary.write_u64::<LittleEndian>(0).unwrap();
            },
            OP::Else => {
                let start = cross_ref_stack.pop().unwrap();
                
                binary.write_u8(Instruction::Jump as u8).unwrap();
                cross_ref_stack.push((op, binary.position()));
                binary.write_u64::<LittleEndian>(0).unwrap();
                
                match start {
                    (OP::If, jump) => {
                        let ret = binary.position();
                        binary.set_position(jump);
                        binary.write_u64::<LittleEndian>(ret).unwrap();
                        binary.set_position(ret);
                    },
                    _ => todo!("else has to follow if"),
                }
            }
            OP::End => {
                let start = cross_ref_stack.pop().unwrap();
                match start {
                    (OP::If, pos) => {
                        let ret = binary.position();
                        binary.set_position(pos);
                        binary.write_u64::<LittleEndian>(ret).unwrap();
                        binary.set_position(ret);
                    },
                    (OP::Else, pos) => {
                        let ret = binary.position();
                        binary.set_position(pos);
                        binary.write_u64::<LittleEndian>(ret).unwrap();
                        binary.set_position(ret);
                    }
                    _ => todo!(),
                }
            },
            OP::Call(func) => {
                binary.write_u8(Instruction::Call as u8).unwrap();
                binary.write_u64::<LittleEndian>(*functions.get(func.as_str()).unwrap()).unwrap();
            },
            OP::Poke => binary.write_u8(Instruction::Poke as u8).unwrap(),
        }
    };
    binary.write_u8(Instruction::Exit as u8).unwrap();

    // ====================[ Data Section ]========================
    let offset = binary.position();
    binary.write_all(data.as_slice()).unwrap();
    binary.set_position(8);
    binary.write_u64::<LittleEndian>(offset).unwrap();


    return binary.into_inner();
}

fn lex(input: &str) -> Vec<OP> {    
    enum Mode {
        Word,
        String,
    }
    let mut mode = Mode::Word;
 
    let mut program = Vec::<OP>::new();

    let mut functions = HashSet::new();
    functions.insert("putd");
    functions.insert("alloc");
    functions.insert("write");

    let mut word = String::new();

    let mut i = 0;
    while let Some(c) = input.chars().nth(i) {
        i += 1;
        
        
        match mode {
            Mode::Word => {
                if c.is_whitespace() {
                    if !word.is_empty() {
                        program.push(lex_word(word.as_str(), &functions));
                        word.clear();
                    }
                }
                else if c == '"' {
                    assert!(word.is_empty());
                    mode = Mode::String;
                }
                else {
                    word.push(c);
                }
            },
            Mode::String => {
                if c == '"' {
                    program.push(OP::PushStr(word.clone()));
                    word.clear();
                    mode = Mode::Word;
                }
                else {
                    word.push(c);
                }
            },
        }
    }

    program.push(lex_word(word.as_str(), &functions));

    return program;
}

fn lex_word(word: &str, functions: &HashSet<&str>) -> OP {
    match word {
        "dup"   =>  OP::Dup,
        "over"  =>  OP::Over,
        "+"     =>  OP::Add,
        "="     =>  OP::Equal,
        "if"    =>  OP::If,
        "else"  =>  OP::Else,
        "end"   =>  OP::End,
        "!"     =>  OP::Poke,
        tok if word.parse::<u64>().is_ok() => OP::Push(tok.parse::<i64>().unwrap()),
        func if functions.contains(func) => OP::Call(word.into()),
        _ => todo!("unknown word `{word}`"),
    }
}