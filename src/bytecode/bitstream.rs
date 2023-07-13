use super::{ThetaFunction, ThetaConstant};


#[derive(Debug)]
pub struct ThetaBitstream {
    constants: Vec<ThetaConstant>,
    functions: Vec<ThetaFunction>,
}