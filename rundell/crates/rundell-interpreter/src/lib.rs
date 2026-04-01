//! Rundell interpreter crate.
//!
//! Provides a tree-walk evaluator for Rundell ASTs produced by
//! `rundell-parser`.

pub mod environment;
pub mod error;
pub mod evaluator;
pub mod form_registry;
pub mod gui_channel;

pub use error::RuntimeError;
pub use evaluator::Interpreter;

#[cfg(test)]
mod tests {
    use super::*;
    use rundell_parser::parse;

    /// Run source code and capture stdout as a String.
    fn run_capture(src: &str) -> String {
        // We collect output via a shared buffer.
        let buf = std::rc::Rc::new(std::cell::RefCell::new(Vec::<u8>::new()));
        let writer = CollectingWriter(buf.clone());
        let mut interp = Interpreter::new_with_output(Box::new(writer));
        let stmts = parse(src).unwrap_or_else(|e| panic!("parse error: {e}\n{src}"));
        interp
            .run(stmts)
            .unwrap_or_else(|e| panic!("runtime error: {e}\n{src}"));
        let bytes = buf.borrow().clone();
        String::from_utf8(bytes).unwrap()
    }

    /// A `Write` implementation that accumulates bytes into a shared buffer.
    struct CollectingWriter(std::rc::Rc<std::cell::RefCell<Vec<u8>>>);

    impl std::io::Write for CollectingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_variables() {
        let out = run_capture(include_str!("../../../tests/01_variables.run"));
        assert_eq!(out, "Rundell\n5\n3.14\n9.99\ntrue\nnothing is null\n");
    }

    #[test]
    fn test_arithmetic() {
        let out = run_capture(include_str!("../../../tests/02_arithmetic.run"));
        assert_eq!(out, "13\n7\n30\n3\n1\n256\n3.3333333333333335\n");
    }

    #[test]
    fn test_strings() {
        let out = run_capture(include_str!("../../../tests/03_strings.run"));
        assert_eq!(
            out,
            "Hello, World!\n13\nHELLO, WORLD!\nhello, world!\nHello\nspaced\n"
        );
    }

    #[test]
    fn test_booleans() {
        let out = run_capture(include_str!("../../../tests/04_boolean.run"));
        assert_eq!(out, "true\nfalse\ntrue\nfalse\nlogic works\n");
    }

    #[test]
    fn test_casting() {
        let out = run_capture(include_str!("../../../tests/05_casting.run"));
        assert_eq!(out, "42.0\n42\nfalse\n");
    }

    #[test]
    fn test_conditionals() {
        let out = run_capture(include_str!("../../../tests/06_conditionals.run"));
        assert_eq!(out, "Distinction\nGrade A\n");
    }

    #[test]
    fn test_loops() {
        let out = run_capture(include_str!("../../../tests/07_loops.run"));
        assert_eq!(out, "1\n2\n3\n1\n2\n3\n");
    }

    #[test]
    fn test_functions() {
        let out = run_capture(include_str!("../../../tests/08_functions.run"));
        assert_eq!(out, "42\nHello, World!\n");
    }

    #[test]
    fn test_collections() {
        let out = run_capture(include_str!("../../../tests/09_collections.run"));
        assert_eq!(out, "Apple\n2\nApple\nBanana\n");
    }

    #[test]
    fn test_error_handling() {
        let out = run_capture(include_str!("../../../tests/10_error_handling.run"));
        assert_eq!(out, "caught null error\nfinally ran\n");
    }
}
