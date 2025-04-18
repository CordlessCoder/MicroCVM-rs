use std::fmt::Display;
use std::fs::File;
use std::io;

use crate::types::Color;

const FREE_MEMORY: usize = 2048 * 1024;
const VIDEO_MEMORY: usize = 1728 * 1024;

pub struct MicroCVMCpu {
    pub memory: Vec<u8>,
    pub video_memory: Vec<super::types::Color>,
    pub registers: [u8; 8],
    pub sp: u8,
    pub pc: u8,
    pub flags: u8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpcodeType {
    Load = 0x01,
    Store = 0x02,
    Add = 0x03,
    Sub = 0x04,
    Jmp = 0x05,
    Hlt = 0xFF,
    Mov = 0x06,
    Inc = 0x07,
    Div = 0x08,
    Mul = 0x09,
    Nop = 0x90,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum VideoOpcodeType {
    Fill = 0x01,
    Clear = 0x02,
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
    R0 = 0x00,
    R1 = 0x01,
    R2 = 0x02,
    R3 = 0x03,
    R4 = 0x04,
    R5 = 0x05,
    R6 = 0x06,
    R7 = 0x07,
}

pub struct Opcode {
    pub opcode_type: OpcodeType,
    pub argument_count: u8,
    pub arg1: Option<OpcodeArg1>,
    pub arg2: Option<OpcodeArg2>,
}

pub struct VideoOpcode {
    pub opcode_type: VideoOpcodeType,
    pub arg1: Option<u8>,
    pub arg2: Option<u8>,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct InvalidOpcode(pub u8);

impl Display for InvalidOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Opcode: {}", self.0)
    }
}

#[derive(Debug)]
pub struct InvalidRegister(pub u8);

impl Display for InvalidRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Register: {}", self.0)
    }
}

pub enum OpcodeArg1 {
    Register(Register),
    Address(u8),
}

pub enum OpcodeArg2 {
    Register(Register),
    Immediate(u8),
    Address(u8),
}

impl MicroCVMCpu {
    pub fn empty() -> Self {
        Self {
            memory: vec![0; FREE_MEMORY],
            video_memory: vec![Color::new(0, 0, 0); VIDEO_MEMORY],
            registers: [0; 8],
            sp: 0,
            pc: 0,
            flags: 0,
        }
    }
    pub fn get_opcode_argument_count(opcode_type: OpcodeType) -> u8 {
        match opcode_type {
            OpcodeType::Inc => 1,
            OpcodeType::Mov => 2,
            OpcodeType::Add => 2,
            OpcodeType::Sub => 2,
            OpcodeType::Div => 2,
            OpcodeType::Mul => 2,
            _ => 0,
        }
    }

    pub fn create_opcode(&mut self) -> Opcode {
        let mut current_instruction = Opcode::empty();

        let opcode_byte: u8 = self.memory[self.pc as usize];
        current_instruction.opcode_type =
            OpcodeType::try_from(opcode_byte).unwrap_or(OpcodeType::Nop);

        current_instruction.argument_count =
            Self::get_opcode_argument_count(current_instruction.opcode_type);

        if current_instruction.argument_count >= 1 {
            let arg1 = self.memory[(self.pc + 1) as usize];
            current_instruction.arg1 = Some(if arg1 < 8 {
                OpcodeArg1::Register(Register::try_from(arg1).unwrap())
            } else {
                OpcodeArg1::Address(arg1)
            });
        }

        if current_instruction.argument_count >= 2 {
            let arg2 = self.memory[(self.pc + 2) as usize];
            current_instruction.arg2 = Some(if arg2 < 8 {
                OpcodeArg2::Register(Register::try_from(arg2).unwrap())
            } else {
                OpcodeArg2::Address(arg2)
            });
        }

        current_instruction
    }

    pub fn execute_instruction(&mut self) {
        let opcode = self.create_opcode();

        match opcode.opcode_type {
            OpcodeType::Inc => {
                if let Some(OpcodeArg1::Register(reg)) = opcode.arg1 {
                    self.registers[reg as usize] += 1;
                }
            }

            OpcodeType::Mov => {
                if let (Some(OpcodeArg1::Register(dst)), Some(OpcodeArg2::Address(imm))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.registers[dst as usize] = imm;
                }
            }

            OpcodeType::Add => {
                if let (Some(OpcodeArg1::Register(dst)), Some(OpcodeArg2::Address(imm))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.registers[dst as usize] += imm;
                }
            }

            OpcodeType::Sub => {
                if let (Some(OpcodeArg1::Register(dst)), Some(OpcodeArg2::Address(imm))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.registers[dst as usize] -= imm;
                }
            }

            OpcodeType::Div => {
                if let (Some(OpcodeArg1::Register(dst)), Some(OpcodeArg2::Address(imm))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.registers[dst as usize] /= imm;
                }
            }

            OpcodeType::Mul => {
                if let (Some(OpcodeArg1::Register(dst)), Some(OpcodeArg2::Address(imm))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.registers[dst as usize] *= imm;
                }
            }

            OpcodeType::Load => {
                if let (Some(OpcodeArg1::Register(dst)), Some(OpcodeArg2::Address(addr))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.registers[dst as usize] = self.memory[addr as usize];
                }
            }

            OpcodeType::Store => {
                if let (Some(OpcodeArg1::Address(addr)), Some(OpcodeArg2::Register(src))) =
                    (opcode.arg1, opcode.arg2)
                {
                    self.memory[addr as usize] = self.registers[src as usize];
                }
            }

            OpcodeType::Jmp => {
                if let Some(OpcodeArg1::Address(target)) = opcode.arg1 {
                    self.pc = target;
                }
            }

            OpcodeType::Nop => {}
            OpcodeType::Hlt => {}
        }
    }

    pub fn read_memory_from_file(&mut self, file_path: &str) -> io::Result<u64> {
        let mut file = File::open(file_path)?;
        self.memory.clear();
        let read = std::io::copy(&mut file, &mut self.memory).unwrap();

        Ok(read)
    }
}

impl Opcode {
    pub fn empty() -> Self {
        Self {
            opcode_type: OpcodeType::Nop,
            argument_count: 0,
            arg1: None,
            arg2: None,
        }
    }
}

impl TryFrom<u8> for OpcodeType {
    type Error = InvalidOpcode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x01 => OpcodeType::Load,
            0x02 => OpcodeType::Store,
            0x03 => OpcodeType::Add,
            0x04 => OpcodeType::Sub,
            0x05 => OpcodeType::Jmp,
            0x06 => OpcodeType::Mov,
            0x07 => OpcodeType::Inc,
            0x08 => OpcodeType::Div,
            0x09 => OpcodeType::Mul,
            0xFF => OpcodeType::Hlt,
            0x90 => OpcodeType::Nop,
            invalid => return Err(InvalidOpcode(invalid)),
        })
    }
}

impl TryFrom<u8> for Register {
    type Error = InvalidRegister;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,
            invalid => return Err(InvalidRegister(invalid)),
        })
    }
}
