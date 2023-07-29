use std::rc::Rc;
use std::ops::{Deref, Add};
use std::fmt::Debug;

use crate::ast::Function;
use crate::ast::transformers::typeck::TypeInformation;
use crate::bytecode::{OpCode, Chunk};

use super::Symbol;

pub const CONSTANT_POOL_HEADER: [u8; 8] = [84, 104, 101, 67, 111, 110, 115, 116];
pub const FUNCTION_POOL_HEADER: [u8; 8] = [0xF4, 0x17, 0xC7, 0x10, 0x17, 0x90, 0x09, 0xF4];
pub const FUNCTION_HEADER: [u8; 4] = [0x11, 0x22, 0x33, 0x44];

pub const DOUBLE_MARKER: &[u8] = &[0xF, 0xF];
pub const INT_MARKER: &[u8] = &[0xA, 0xA];
pub const BOOL_MARKER: &[u8] = &[0xB, 0xB];
pub const STRING_MARKER: &[u8] = &[0xC, 0xC];

#[derive(Debug, Clone)]
pub enum ThetaValue {
    Double(f64),
    Int(i64),
    Bool(bool),
    HeapValue(Rc<ThetaHeapValue>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ThetaConstant {
    Double(f64),
    Int(i64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Clone)]
pub struct ThetaString {
    internal: Rc<String>,
}

impl ThetaString {
    pub fn new(st: String) -> ThetaString {
        ThetaString { internal: Rc::new(st) }
    }

    pub fn internal(&self) -> &Rc<String> {
        &self.internal
    }
}

impl From<Symbol> for ThetaString {
    fn from(value: Symbol) -> Self {
        ThetaString::new(value.to_string())
    }
}

impl Add<&ThetaString> for ThetaString {
    type Output = ThetaString;

    fn add(self, rhs: &ThetaString) -> Self::Output {
        let s: String = self.internal.to_string() + rhs.as_str();
        ThetaString::new(s)
    }
}

impl Deref for ThetaString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.internal.as_ref()
    }

    
}

pub struct ThetaUserType {

}

#[derive(Debug, Clone)]
pub struct ThetaFunction {
    pub args: Vec<ThetaFuncArg>,
    pub chunk: Chunk,
    pub name: ThetaString,
    pub return_ty: TypeInformation,
}

#[derive(Debug, Clone)]
pub struct ThetaCompiledFunction {
    pub args: Vec<ThetaFuncArg>,
    pub chunk: Vec<u8>,
    pub name: ThetaString,
    pub return_ty: TypeInformation,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ThetaFuncArg {
    pub ty: TypeInformation
}

impl From<TypeInformation> for ThetaFuncArg {
    fn from(value: TypeInformation) -> Self {
        ThetaFuncArg { ty: value }
    }
}

#[derive(Debug, Clone)]

pub enum ThetaHeapValue {
    Str(ThetaString),
}