use log::debug;

use crate::{bytecode::{ThetaValue, ThetaCompiledBitstream, ThetaString, ThetaCompiledFunction, ThetaFileVisitor, ThetaConstant, ThetaFileWalker}};

use super::{Disassembler, DisassembleError};

pub struct BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    interning_fn: &'a mut F,
    bitstream: ThetaCompiledBitstream,
}

impl<'a, F> BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    pub fn new(interning_fn: &'a mut F) -> BasicDisassembler<'a, F> {
        BasicDisassembler {
            interning_fn,
            bitstream: ThetaCompiledBitstream::new()
        }
    }
}

impl<'a, F> Disassembler for BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    type Out = ThetaCompiledBitstream;

    fn disassemble(&mut self, input: &dyn AsRef<[u8]>) -> Result<ThetaCompiledBitstream, DisassembleError> {
        // drive disassembly
        let mut tfw = ThetaFileWalker {};
        tfw.walk_theta_file(self, input)?;
        
        Ok(self.bitstream.clone())
    }
}

impl<'a, F> ThetaFileVisitor for BasicDisassembler<'a, F> where F: FnMut(ThetaString) -> ThetaValue {
    fn visit_theta_file(&mut self) {
        debug!("seen theta file")
    }

    fn visit_theta_bitstream(&mut self) {
        debug!("seen theta bitstream");
        self.bitstream = ThetaCompiledBitstream::new();
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

    fn visit_theta_function(&mut self, function: ThetaCompiledFunction) {
        self.bitstream.functions.push(function);
    }
}