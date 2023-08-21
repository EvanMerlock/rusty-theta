use log::debug;

use crate::bytecode::{
    CHUNK_HEADER, ThetaConstant, ThetaFileVisitor,
};

use super::{DisassembleError, Disassembler};

pub struct StringDisassembler {
    readout: String,
}

impl StringDisassembler {
    pub fn new() -> StringDisassembler {
        StringDisassembler {
            readout: String::new()
        }
    }

    fn disassemble_chunk(&mut self, chunk: &[u8]) -> Result<(usize, String), DisassembleError> {
        let mut offset = 18;
        let mut readout = String::new();

        println!("chunk: {:?}", chunk);

        // assert chunk header
        assert!(chunk[0..8] == CHUNK_HEADER);

        readout.push_str("=== BEGIN CHUNK ===\r\n");

        while offset < chunk.len() {
            // read into chunk
            match chunk[offset] {
                0x0 => {
                    readout.push_str("Op: Return Void (0x0)\r\n");
                    offset += 1
                },
                0xF0 => {
                    readout.push_str("Op: Return (0xF0)\r\n");
                    offset += 1
                }
                0x1 => {
                    readout.push_str(&format!(
                        "Op: Constant (0x1) with offset: {}\r\n",
                        &chunk[offset + 1]
                    ));
                    offset += 2
                }
                0x2 => {
                    readout.push_str(&format!("Op: Push (0x2) with offset: {:?}\r\n",
                        &chunk[offset + 1..offset + 1 + std::mem::size_of::<usize>()]
                    ));
                    offset += 1 + std::mem::size_of::<usize>()
                }
                0x3 => {
                    readout.push_str("Op: Pop (0x3)\r\n");
                    offset += 1
                }
                0x4 => {
                    readout.push_str("Op: Add (0x4)\r\n");
                    offset += 1
                }
                0x5 => {
                    readout.push_str("Op: Sub (0x5)\r\n");
                    offset += 1
                }
                0x6 => {
                    readout.push_str("Op: Mul (0x6)\r\n");
                    offset += 1
                }
                0x7 => {
                    readout.push_str("Op: Div (0x7)\r\n");
                    offset += 1
                }
                0x8 => {
                    readout.push_str("Op: Neg (0x8)\r\n");
                    offset += 1
                }
                0x9 => {
                    readout.push_str("Op: Eq (0x9)\r\n");
                    offset += 1
                }
                0xA => {
                    readout.push_str("Op: GT (0xA)\r\n");
                    offset += 1
                },
                0xA1 => {
                    readout.push_str("Op: GTE (0xA1)\r\n");
                    offset += 1
                }
                0xB => {
                    readout.push_str("Op: LT (0xB)\r\n");
                    offset += 1
                },
                0xB1 => {
                    readout.push_str("Op: LTE (0xB1)\r\n");
                    offset += 1
                }
                0xC0 => {
                    readout.push_str(&format!(
                        "Op: Def Global (0x1) with offset: {}\r\n",
                        &chunk[offset + 1]
                    ));
                    offset += 2
                }
                0xC1 => {
                    readout.push_str(&format!(
                        "Op: Get Global (0x1) with offset: {}\r\n",
                        &chunk[offset + 1]
                    ));
                    offset += 2
                }
                0xD0 => {
                    readout.push_str(&format!(
                        "Op: Jump Unconditional (0xD0) with offset: {}\r\n",
                        &chunk[offset + 1]
                    ));
                    offset += 2
                }
                0xD1 => {
                    readout.push_str(&format!(
                        "Op: Jump If False (0xD1) with offset: {}\r\n",
                        &chunk[offset + 1]
                    ));
                    offset += 2
                }
                0xD2 => {
                    readout.push_str(&format!(
                        "Op: Jump Unconditional Far (0xD2) with offset: {:?}\r\n",
                        &chunk[offset + 1..offset + 1 + std::mem::size_of::<isize>()]
                    ));
                    offset += 1 + std::mem::size_of::<isize>()
                }
                0xD3 => {
                    readout.push_str(&format!(
                        "Op: Jump Far If False (0xD3) with offset: {:?}\r\n",
                        &chunk[offset + 1..offset + 1 + std::mem::size_of::<isize>()]
                    ));
                    offset += 1 + std::mem::size_of::<isize>()
                },
                0xE0 => {
                    readout.push_str(&format!(
                        "Op: Call Direct (0xE0) with offset: {}\r\n",
                        &chunk[offset + 1]
                    ));
                    offset += 2
                },
                0xFE => {
                    readout.push_str("Op: Breakpoint (0xFE)\r\n");
                    offset += 1
                }
                0xFD => {
                    readout.push_str("Op: Noop (0xFD)\r\n");
                    offset += 1
                }
                0xFF => {
                    readout.push_str("Op: Print (0xFF)\r\n");
                    offset += 1
                }
                code => {
                    readout.push_str(&format!("Op: Unknown ({:#x})\r\n", code));
                    offset += 1
                }
            }
        }

        Ok((0, readout))
    }
}

impl Default for StringDisassembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Disassembler for StringDisassembler {
    type Out = String;

    fn disassemble(&mut self, input: &dyn AsRef<[u8]>) -> Result<String, DisassembleError> {
        let mut readout = String::new();

        // TOOD: this only handles 1 chunk as that's all we're passing it right now.
        let (_offset, chunk_disassembly) = self.disassemble_chunk(input.as_ref())?;
        readout.push_str(&chunk_disassembly);

        Ok(readout)
    }

}

impl ThetaFileVisitor for StringDisassembler {
    fn visit_theta_file(&mut self) {
        debug!("seen theta file")
    }

    fn visit_theta_bitstream(&mut self) {
        debug!("seen theta bitstream");
        self.readout = String::new();
    }

    fn visit_theta_constant(&mut self, constant: ThetaConstant) {
        debug!("seen theta constant");
        match constant {
            ThetaConstant::Double(dbl) => self.readout.push_str(&format!("Constant: {}\r\n", dbl)),
            ThetaConstant::Int(int) => self.readout.push_str(&format!("Constant: {}\r\n", int)),
            ThetaConstant::Bool(bln) => self.readout.push_str(&format!("Constant: {}\r\n", bln)),
            ThetaConstant::Str(strin) => self.readout.push_str(&format!("Constant: {}\r\n", strin)),
        }
    }

    fn visit_theta_function(&mut self, _function: crate::bytecode::ThetaCompiledFunction) {
        debug!("seen theta function")
    }
}