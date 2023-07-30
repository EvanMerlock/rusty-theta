const IF_CONDITION: &'static str = "if (true) { let y: Int = 4; print(y); } else { let y: Int = 5; print(y); };";
const WHILE_INFINITE: &'static str = "while { print(\"hello, world\"); };";
const WHILE_INFINITE_FN: &'static str = "fun test() { print(\"hello, world\"); } \r\n while { test(); };";