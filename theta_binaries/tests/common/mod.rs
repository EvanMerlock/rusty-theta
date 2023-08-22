use std::{rc::Rc, io::Write, cell::RefCell};

use theta_vm::vm::VM;


use theta_compiler::{lexer::{BasicLexer, Lexer}, parser::{BasicParser, Parser}, ast::{symbol::{ExtSymbolTable, SymbolData}, transformers::{typeck::TypeCk, ASTTransformer, to_bytecode::ToByteCode}, Item}};
use theta_types::{bytecode::{ThetaBitstream, ThetaConstant, OpCode, BasicDisassembler, Disassembler, BasicAssembler, Assembler, ThetaCompiledBitstream, Chunk}, types::TypeInformation, build_chunk};

#[derive(Clone)]
pub struct TestOutput {
    pub inner: Rc<RefCell<Vec<u8>>>
}

impl TestOutput {
    pub fn new() -> TestOutput {
        TestOutput { inner: Rc::new(RefCell::new(Vec::default())) }
    }
}

impl Write for TestOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.borrow_mut().flush()
    }
}

pub fn build_test_vm(code: &'static str, fn_name: &'static str, fn_transform: impl Fn(Chunk) -> Chunk, stdout: Box<dyn Write>) -> Result<(VM, Rc<ThetaCompiledBitstream>, Vec<u8>), Box<dyn std::error::Error>> { 

    let tbl = ExtSymbolTable::default();
    let mut machine = VM::new(stdout);

    let mut chars = code.chars();
    let lexer = BasicLexer::new(&mut chars);
    let tokens = lexer.lex()?;
    let parser = BasicParser::new_sym(tokens.output(), tbl.clone());
    let trees = parser.parse()?;

    let mut bitstream = ThetaBitstream::new();

    for item in trees {
        let sym = &item.information().current_symbol_table;
        let type_cker = TypeCk::new(sym.clone());
        let type_check = type_cker.transform_item(&item)?;

        match item {
            Item::Function(func) => {
                let mut tbl =  tbl.borrow_mut();
                tbl.insert_symbol(func.name, SymbolData::Function { 
                    return_ty: func.return_ty.clone(), 
                    args: func.args.clone(), 
                    fn_ty: TypeInformation::Function(Box::new(func.return_ty.clone()), func.args.into_iter().map(|x| x.ty).collect())
                });
            },
        };

        let tbc = ToByteCode::new(tokens.line_mapping());
        let mut theta_func = tbc.transform_item(&type_check)?;

        // need to pull out constants and reloc
        let consts = theta_func.chunk.constants();

        let reloc = bitstream.constants.len();
        bitstream.constants.extend_from_slice(consts);
        
        let new_ck = theta_func.chunk.relocate(reloc);
        theta_func.chunk = new_ck;
        bitstream.functions.push(theta_func);
    }

    let call_function_chunk = build_chunk!(OpCode::Constant { offset: 0 }, OpCode::CallDirect { name_offset: 0 }; ThetaConstant::Str(String::from(fn_name)));
    let reloc = bitstream.constants.len();
    let call_function_chunk = call_function_chunk.relocate(reloc);

    bitstream.constants.extend_from_slice(call_function_chunk.constants());

    let mut compiled_bitstream = Vec::new();
    let mut basic_assembler = BasicAssembler::new(&mut compiled_bitstream);
    basic_assembler.assemble_bitstream(bitstream)?;

    let mut intern_fn = |x| machine.intern_string(x);
    let mut basic_diassembler = BasicDisassembler::new(&mut intern_fn);
    let comp_bs = basic_diassembler.disassemble(&compiled_bitstream)?;
    let loaded_bs = machine.load_bitstream(comp_bs);

    let mut compiled_chunk = Vec::new();
    let mut basic_assembler = BasicAssembler::new(&mut compiled_chunk);
    basic_assembler.assemble_chunk(call_function_chunk)?;

    Ok((machine, loaded_bs, compiled_chunk))
}

pub fn identity<T>(x: T) -> T {
    x
}