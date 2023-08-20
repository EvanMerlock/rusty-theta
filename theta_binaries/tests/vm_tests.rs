use theta_types::bytecode::ThetaValue;

const IF_CONDITION: &'static str = "if (true) { let y: Int = 4; print(y); } else { let y: Int = 5; print(y); };";
const WHILE_INFINITE: &'static str = "while { print(\"hello, world\"); };";
const WHILE_INFINITE_FN: &'static str = "fun test() { print(\"hello, world\"); } \r\n while { test(); };";

mod common;

#[test]
pub fn fibbonaci_test() -> Result<(), Box<dyn std::error::Error>> {
    use std::rc::Rc;
    use theta_vm::vm::ThetaCallFrame;

    let code = 
    "fun fib(n: Int) -> Int {
        if (n <= 1) {
            n
        } else {
            fib(n-1) + fib(n-2)
        }
    }";

    let stdout = common::TestOutput::new();

    let (mut machine, loaded_bs, compiled_chunk) = crate::common::build_test_vm(code, Box::new(stdout.clone()))?;

    // do the magic stack frame thing
    machine.push_frame(ThetaCallFrame { rip: 0, locals: vec![Some(ThetaValue::Int(10))], bitstream: loaded_bs, chunk: Rc::new(compiled_chunk) });

    // execute chunk
    machine.execute_code()?;
    
    let output = stdout.inner.borrow();
    assert_eq!(output.as_slice(), &[]);
    assert_eq!(machine.stack().curr_frame().expect("failed to get stack").locals.last().expect("nothing on top of stack").clone().expect("nothing on top of stack").clone(), ThetaValue::Int(55));


    Ok(())

}