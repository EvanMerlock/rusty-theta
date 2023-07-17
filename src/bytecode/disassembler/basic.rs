use log::debug;

use crate::{bytecode::{BITSTREAM_HEADER, CONSTANT_POOL_HEADER, DOUBLE_MARKER, ThetaValue, INT_MARKER, BOOL_MARKER, STRING_MARKER, OpCode, ThetaBitstream, ThetaString, ThetaHeapValue, FUNCTION_POOL_HEADER, FUNCTION_HEADER, ThetaFunction, ThetaFuncArg, CHUNK_HEADER, ThetaFileVisitor, ThetaConstant}, ast::transformers::typeck::TypeInformation};

use super::{Disassembler, DisassembleError};

pub struct BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    interning_fn: &'a mut F,
    bitstream: ThetaBitstream,
}

impl<'a, F> BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    pub fn new(interning_fn: &'a mut F) -> BasicDisassembler<'a, F> {
        BasicDisassembler {
            interning_fn,
            bitstream: ThetaBitstream::new()
        }
    }
}

impl<'a, F> Disassembler for BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    type Out = ThetaBitstream;

    fn disassemble(&mut self, input: &dyn AsRef<[u8]>) -> Result<ThetaBitstream, DisassembleError> {
        Ok(self.bitstream.clone())
    }
}

impl<'a, F> ThetaFileVisitor for BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    fn visit_theta_file(&mut self) {
        debug!("seen theta file")
    }

    fn visit_theta_bitstream(&mut self) {
        debug!("seen theta bitstream");
        self.bitstream = ThetaBitstream::new();
    }

    fn visit_theta_constant(&mut self, constant: ThetaConstant) {
        debug!("seen theta constant");
        match constant {
            ThetaConstant::Double(dbl) => self.bitstream.constants.push(ThetaValue::Double(dbl)),
            ThetaConstant::Int(int) => self.bitstream.constants.push(ThetaValue::Int(int)),
            ThetaConstant::Bool(bln) => self.bitstream.constants.push(ThetaValue::Bool(bln)),
            ThetaConstant::Str(strin) => {
                debug!("attempting to intern string");
                let interned = (self.interning_fn)(ThetaString::new(strin));
                self.bitstream.constants.push(interned);
            },
        }
    }

    fn visit_theta_function(&mut self, function: ThetaFunction) {
        todo!()
    }
}