use std::{fmt, error::Error};

use log::debug;

use crate::{bytecode::{BITSTREAM_HEADER, CONSTANT_POOL_HEADER, DOUBLE_MARKER, INT_MARKER, BOOL_MARKER, STRING_MARKER, ThetaString, FUNCTION_POOL_HEADER, FUNCTION_HEADER, ThetaCompiledFunction, ThetaFuncArg, CHUNK_HEADER}, ast::transformers::typeck::TypeInformation};

use super::ThetaConstant;

pub trait ThetaFileVisitor {
    fn visit_theta_file(&mut self);
    fn visit_theta_bitstream(&mut self);
    fn visit_theta_constant(&mut self, constant: ThetaConstant);
    fn visit_theta_function(&mut self, function: ThetaCompiledFunction);
}

pub struct ThetaFileWalker {}

#[derive(Debug)]
pub enum FileVisitError {
    IOError(std::io::Error),
    TryFromSliceError(std::array::TryFromSliceError),
    Utf8Error(std::string::FromUtf8Error),
    InvalidMarkerInChunk(Vec<u8>),
}

impl fmt::Display for FileVisitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileVisitError::IOError(io) => write!(f, "I/O error: {}", io),
            FileVisitError::TryFromSliceError(tfs) => write!(f, "Could not get item from slice: {}", tfs),
            FileVisitError::Utf8Error(utf) => write!(f, "UTF-8 error: {}", utf),
            FileVisitError::InvalidMarkerInChunk(marker) => write!(f, "invalid marker: [{}, {}]", marker[0], marker[1]),
        }
    }
}

impl Error for FileVisitError {}

impl From<std::io::Error> for FileVisitError {
    fn from(err: std::io::Error) -> Self {
        FileVisitError::IOError(err)
    }
}

impl From<std::array::TryFromSliceError> for FileVisitError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        FileVisitError::TryFromSliceError(err)
    }
}

impl From<std::string::FromUtf8Error> for FileVisitError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        FileVisitError::Utf8Error(e)
    }
}

impl ThetaFileWalker {
    pub fn walk_theta_file(&mut self, visitor: &mut dyn ThetaFileVisitor, input: &dyn AsRef<[u8]>) -> Result<(), FileVisitError> {
        visitor.visit_theta_file();
        self.walk_bitstream(visitor, input.as_ref())?;
        Ok(())
    }

    fn walk_bitstream(&mut self, visitor: &mut dyn ThetaFileVisitor, bitstream: &[u8]) -> Result<(), FileVisitError> {
        // assert bitstream header
        assert!(bitstream[0..8] == BITSTREAM_HEADER);
        debug!("=== BEGIN BITSTREAM ===");
        visitor.visit_theta_bitstream();

        // first segment of the bitstream is the constant pool
        let fn_offset = self.walk_constant_pool(visitor, &bitstream[8..])?;
        let _after_offset = self.walk_function_pool(visitor, &bitstream[8+fn_offset..])?;

        Ok(())
    }

    fn walk_constant_pool(&mut self, visitor: &mut dyn ThetaFileVisitor, constant_pool: &[u8]) -> Result<usize, FileVisitError> {
        // assert constant pool header
        assert!(constant_pool[0..8] == CONSTANT_POOL_HEADER);

        debug!("-- BEGIN CONSTANT POOL --");

        // read const pool size
        // u16 on disk, only looking at byte 9
        // TODO: all sizes on disk should be usize.
        let const_pool_size = constant_pool[9];
        let mut offset = 10;

        for _ in 0..const_pool_size {
            let marker = &constant_pool[offset..offset+2];
            debug!("marker: {:?}", marker);
            match marker {
                sli if sli == DOUBLE_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = constant_pool[offset..offset+8].try_into()?;
                    let float = f64::from_le_bytes(dbl);
                    debug!("float found in constant pool: {}", float);
                    visitor.visit_theta_constant(ThetaConstant::Double(float));
                    offset += 8;
                },
                sli if sli == INT_MARKER => {
                    offset += 2;
                    let dbl: [u8; 8] = constant_pool[offset..offset+8].try_into()?;
                    let int = i64::from_le_bytes(dbl);
                    debug!("i64 found in constant pool: {}", int);
                    visitor.visit_theta_constant(ThetaConstant::Int(int));
                    offset += 8;
                },
                sli if sli == BOOL_MARKER => {
                    offset += 2;
                    let bol: [u8; 1] = constant_pool[offset..offset+1].try_into()?;
                    let bol = bol == [1u8];
                    debug!("bool found in constant pool: {}", bol);
                    visitor.visit_theta_constant(ThetaConstant::Bool(bol));
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
                    visitor.visit_theta_constant(ThetaConstant::Str(read_str));
                    offset += len;
                }
                _ => return Err(FileVisitError::InvalidMarkerInChunk(marker.to_vec())),
            }
        }
        Ok(offset)
    }

    fn walk_function_pool(&mut self, visitor: &mut dyn ThetaFileVisitor, function_pool: &[u8]) -> Result<usize, FileVisitError> {
        // assert function pool header
        assert!(function_pool[0..8] == FUNCTION_POOL_HEADER);

        debug!("-- BEGIN FUNCTION POOL --");
        let func_pool_size = function_pool[8];
        let mut offset = 9;

        for _ in 0..func_pool_size {
            debug!("Fn found");

            // assert header
            assert!(function_pool[offset..offset+4] == FUNCTION_HEADER);
            offset+=4;

            debug!("reading in fn name");
            let fn_name_size: usize = usize::from_le_bytes(function_pool[offset..offset+8].try_into().expect("could not get fn name size"));
            offset += 8;
            // TODO: this can overflow very easily with a cleverly crafted file.
            let fn_name = String::from_utf8(function_pool[offset..offset+fn_name_size].to_vec()).expect("could not get fn name");

            debug!("function named: {fn_name}");

            offset += fn_name_size;
            debug!("reading fn args");

            // arity of fn
            let fn_arity: usize = usize::from_le_bytes(function_pool[offset..offset+8].try_into().expect("could not get fn arity"));
            offset += 8;

            let mut fn_args = vec![];
            for _ in 0..fn_arity {
                match function_pool[offset] {
                    0x0 => fn_args.push(ThetaFuncArg::from(TypeInformation::None)),
                    0x1 => fn_args.push(ThetaFuncArg::from(TypeInformation::Boolean)),
                    0x2 => fn_args.push(ThetaFuncArg::from(TypeInformation::Int)),
                    0x3 => fn_args.push(ThetaFuncArg::from(TypeInformation::Float)),
                    0x4 => fn_args.push(ThetaFuncArg::from(TypeInformation::String)),
                    _ => panic!("unknown ty info")
                }
                offset += 1;
            }

            debug!("reading fn return type");
            let fn_return_ty = match function_pool[offset] {
                0x0 => TypeInformation::None,
                0x1 => TypeInformation::Boolean,
                0x2 => TypeInformation::Int,
                0x3 => TypeInformation::Float,
                0x4 => TypeInformation::String,
                _ => panic!("unknown ty info")
            };
            offset += 1;

            debug!("reading fn bitstream");
            let (new_off, chunk_code) = self.walk_chunk(&function_pool[offset..])?;

            visitor.visit_theta_function(ThetaCompiledFunction {
                args: fn_args,
                chunk: chunk_code,
                name: ThetaString::new(fn_name),
                return_ty: fn_return_ty,
            });

            offset = new_off;
        }

        Ok(offset)

    }

    fn walk_chunk(&mut self, chunk: &[u8]) -> Result<(usize, Vec<u8>), FileVisitError> {
        debug!("-- BEGIN CHUNK --");

        assert!(chunk[0..8] == CHUNK_HEADER);
        let mut offset = 8;
        let chunk_size: usize = usize::from_le_bytes(chunk[offset..offset+8].try_into().expect("could not get chunk size"));
        offset += 7;

        debug!("chunk size: {chunk_size}");

        Ok((offset, chunk[offset..offset+chunk_size].to_vec()))
    }
}