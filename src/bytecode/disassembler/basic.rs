use log::debug;

use crate::bytecode::{BITSTREAM_HEADER, CONSTANT_POOL_HEADER, DOUBLE_MARKER, ThetaValue, INT_MARKER, BOOL_MARKER, STRING_MARKER, OpCode, ThetaBitstream, ThetaString, ThetaHeapValue};

use super::{Disassembler, DisassembleError};

pub struct BasicDisassembler {
}

impl BasicDisassembler  {
    pub fn new() -> BasicDisassembler {
        BasicDisassembler {}
    }
}

impl Disassembler for BasicDisassembler  {
    type Out = ThetaBitstream;

    fn disassemble_chunk(&mut self, chunk: &[u8]) -> Result<Vec<OpCode>, DisassembleError> {
        Ok(())
    }

    fn disassemble(&mut self, input: &dyn AsRef<[u8]>) -> Result<ThetaBitstream, DisassembleError> {
        // TOOD: this only handles 1 chunk as that's all we're passing it right now.
        self.disassemble_chunk(input.as_ref())?;

        Ok(())
    }

    fn disassemble_bitstream(&mut self, bitstream: &[u8]) -> Result<ThetaBitstream, DisassembleError> {
        // assert bitstream header
        assert!(bitstream[0..8] == BITSTREAM_HEADER);
        debug!("=== BEGIN BITSTREAM ===");

        // first segment of the bitstream is the constant pool
        self.disassemble_constant_pool(&bitstream[8..])?;


        todo!()
    }

    fn disassemble_constant_pool(&mut self, constant_pool: &[u8]) -> Result<Vec<ThetaValue>, DisassembleError> {

        // assert constant pool header
        assert!(constant_pool[0..8] == CONSTANT_POOL_HEADER);

        debug!("-- BEGIN CONSTANT POOL --");

        // read const pool size
        let const_pool_size = constant_pool[9];
        let mut offset = 10;
        // TODO: constant pool should not be loaded into the VM.
        // Instead, only global variables should be loaded into the global scope.
        // Constants being loaded into the VM would require relocation upon load time
        // However, it may be possible to do this for string interning.
        // https://stackoverflow.com/questions/10578984/what-is-java-string-interning
        // We should still store the string in the constant pool, but if the string literal exists in the heap already
        // We should reference that instead; live bytecode patching could be a possibility, rather than using the same
        // OP_CONSTANT bytecode fragment
        for _ in 0..const_pool_size {
            let marker = &constant_pool[offset..offset+2];
            debug!("marker: {:?}", marker);
            match marker {
                sli if sli == DOUBLE_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = constant_pool[offset..offset+8].try_into()?;
                    let float = f64::from_le_bytes(dbl);
                    debug!("float found in constant pool: {}", float);
                    self.constants.push(ThetaValue::Double(float));              
                    offset += 8;
                },
                sli if sli == INT_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = constant_pool[offset..offset+8].try_into()?;
                    let int = i64::from_le_bytes(dbl);
                    debug!("i64 found in constant pool: {}", int);
                    self.constants.push(ThetaValue::Int(int));              
                    offset += 8;
                },
                sli if sli == BOOL_MARKER => {
                    offset += 2;
                    let bol: [u8; 1] = constant_pool[offset..offset+1].try_into()?;
                    let bol = bol == [1u8];
                    debug!("bool found in constant pool: {}", bol);
                    self.constants.push(ThetaValue::Bool(bol));              
                    offset += 1;
                },
                sli if sli == STRING_MARKER => {
                    offset += 2;
                    let len_bytes: [u8; 8] = constant_pool[offset..offset+8].try_into()?;
                    let len = usize::from_le_bytes(len_bytes);
                    offset += 8;
                    let in_str = &constant_pool[offset..offset+len];
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(in_str);
                    let read_str = String::from_utf8(bytes)?;
                    debug!("str found in constant pool: {}", read_str);
                    debug!("checking for memoized string");
                    let s_val = ThetaString::new(read_str);
                    let tv = (self.interning_func)(s_val);
                    offset += len;
                    self.constants.push(tv);
                }
                _ => return Err(DisassembleError::InvalidMarkerInChunk(marker.to_vec())),
            }
        }
        todo!()
    }
}