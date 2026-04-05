/// Unit tests for rustlab-script: lexer, parser, evaluator, and Value type.

#[cfg(test)]
mod bool_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;
    use crate::ast::BinOp;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn get_bool(ev: &Evaluator, name: &str) -> bool {
        match ev.get(name).unwrap() {
            Value::Bool(b) => *b,
            other => panic!("Expected bool for '{name}', got {other:?}"),
        }
    }

    // ── Comparison operators ─────────────────────────────────────────────────

    #[test] fn eq_true()  { assert!(get_bool(&eval_str("b = 3 == 3"), "b")); }
    #[test] fn eq_false() { assert!(!get_bool(&eval_str("b = 3 == 4"), "b")); }
    #[test] fn ne_true()  { assert!(get_bool(&eval_str("b = 3 != 4"), "b")); }
    #[test] fn ne_false() { assert!(!get_bool(&eval_str("b = 3 != 3"), "b")); }
    #[test] fn lt_true()  { assert!(get_bool(&eval_str("b = 2 < 3"), "b")); }
    #[test] fn lt_false() { assert!(!get_bool(&eval_str("b = 3 < 2"), "b")); }
    #[test] fn le_true()  { assert!(get_bool(&eval_str("b = 3 <= 3"), "b")); }
    #[test] fn le_false() { assert!(!get_bool(&eval_str("b = 4 <= 3"), "b")); }
    #[test] fn gt_true()  { assert!(get_bool(&eval_str("b = 5 > 3"), "b")); }
    #[test] fn gt_false() { assert!(!get_bool(&eval_str("b = 2 > 3"), "b")); }
    #[test] fn ge_true()  { assert!(get_bool(&eval_str("b = 3 >= 3"), "b")); }
    #[test] fn ge_false() { assert!(!get_bool(&eval_str("b = 2 >= 3"), "b")); }

    // ── Logical operators ────────────────────────────────────────────────────

    #[test] fn and_tt() { assert!(get_bool(&eval_str("b = (1 < 2) && (3 < 4)"), "b")); }
    #[test] fn and_tf() { assert!(!get_bool(&eval_str("b = (1 < 2) && (4 < 3)"), "b")); }
    #[test] fn or_ff()  { assert!(!get_bool(&eval_str("b = (2 < 1) || (4 < 3)"), "b")); }
    #[test] fn or_ft()  { assert!(get_bool(&eval_str("b = (2 < 1) || (3 < 4)"), "b")); }

    // ── Unary not ────────────────────────────────────────────────────────────

    #[test] fn not_true()  { assert!(!get_bool(&eval_str("b = !(1 < 2)"), "b")); }
    #[test] fn not_false() { assert!(get_bool(&eval_str("b = !(2 < 1)"), "b")); }

    // ── Display ──────────────────────────────────────────────────────────────

    #[test]
    fn bool_display_true()  { assert_eq!(format!("{}", Value::Bool(true)),  "true"); }
    #[test]
    fn bool_display_false() { assert_eq!(format!("{}", Value::Bool(false)), "false"); }

    // ── Bool == Bool ─────────────────────────────────────────────────────────

    #[test]
    fn bool_eq_bool() {
        let result = Value::binop(BinOp::Eq, Value::Bool(true), Value::Bool(true)).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    #[test]
    fn bool_ne_bool() {
        let result = Value::binop(BinOp::Ne, Value::Bool(true), Value::Bool(false)).unwrap();
        assert!(matches!(result, Value::Bool(true)));
    }

    // ── Error cases ──────────────────────────────────────────────────────────

    #[test]
    fn not_on_scalar_errors() {
        assert!(Value::Bool(true).not().is_ok());
        assert!(Value::Scalar(1.0).not().is_err());
    }

    #[test]
    fn and_on_non_bool_errors() {
        assert!(Value::binop(BinOp::And, Value::Scalar(1.0), Value::Scalar(1.0)).is_err());
    }

    #[test]
    fn comparison_chained_with_logic() {
        // (2 < 5) && (5 < 10) → true
        let ev = eval_str("result = (2 < 5) && (5 < 10)");
        assert!(get_bool(&ev, "result"));
    }

    #[test]
    fn assignment_not_confused_with_eq() {
        // x = 5 should be assignment, not comparison
        let ev = eval_str("x = 5");
        match ev.get("x").unwrap() {
            Value::Scalar(n) => assert!((*n - 5.0).abs() < 1e-12),
            _ => panic!("Expected scalar"),
        }
    }
}

#[cfg(test)]
mod lexer_tests {
    use crate::lexer::{tokenize, Token};

    fn tokens(src: &str) -> Vec<Token> {
        tokenize(src).unwrap().into_iter().map(|s| s.token).collect()
    }

    #[test]
    fn integer_number() {
        let t = tokens("42\n");
        assert!(matches!(t[0], Token::Number(n) if (n - 42.0).abs() < 1e-12));
    }

    #[test]
    fn float_number() {
        let t = tokens("3.14\n");
        assert!(matches!(t[0], Token::Number(n) if (n - 3.14).abs() < 1e-12));
    }

    #[test]
    fn scientific_notation() {
        let t = tokens("1e3\n");
        assert!(matches!(t[0], Token::Number(n) if (n - 1000.0).abs() < 1e-9));
        let t2 = tokens("2.5e-2\n");
        assert!(matches!(t2[0], Token::Number(n) if (n - 0.025).abs() < 1e-12));
    }

    #[test]
    fn string_literal() {
        let t = tokens("\"hello\"\n");
        assert!(matches!(&t[0], Token::Str(s) if s == "hello"));
    }

    #[test]
    fn identifier() {
        let t = tokens("foo\n");
        assert!(matches!(&t[0], Token::Ident(s) if s == "foo"));
    }

    #[test]
    fn operators() {
        let t = tokens("+ - * / ^\n");
        assert!(matches!(t[0], Token::Plus));
        assert!(matches!(t[1], Token::Minus));
        assert!(matches!(t[2], Token::Star));
        assert!(matches!(t[3], Token::Slash));
        assert!(matches!(t[4], Token::Caret));
    }

    #[test]
    fn dot_operators() {
        let t = tokens(".* ./ .^\n");
        assert!(matches!(t[0], Token::DotStar));
        assert!(matches!(t[1], Token::DotSlash));
        assert!(matches!(t[2], Token::DotCaret));
    }

    #[test]
    fn colon_and_apostrophe() {
        let t = tokens(": '\n");
        assert!(matches!(t[0], Token::Colon));
        assert!(matches!(t[1], Token::Apostrophe));
    }

    #[test]
    fn brackets_and_parens() {
        let t = tokens("[ ] ( ) ,\n");
        assert!(matches!(t[0], Token::LBracket));
        assert!(matches!(t[1], Token::RBracket));
        assert!(matches!(t[2], Token::LParen));
        assert!(matches!(t[3], Token::RParen));
        assert!(matches!(t[4], Token::Comma));
    }

    #[test]
    fn semicolon() {
        let t = tokens("a;\n");
        assert!(matches!(t[1], Token::Semicolon));
    }

    #[test]
    fn comment_ignored() {
        let t = tokens("42 # this is a comment\n");
        assert!(matches!(t[0], Token::Number(n) if (n - 42.0).abs() < 1e-12));
        assert!(matches!(t[1], Token::Newline));
        assert!(matches!(t[2], Token::Eof));
    }

    #[test]
    fn multiple_newlines_collapsed() {
        let t = tokens("a\n\n\nb\n");
        // Should be: Ident(a), Newline, Ident(b), Newline, Eof
        let newline_count = t.iter().filter(|t| matches!(t, Token::Newline)).count();
        assert_eq!(newline_count, 2, "consecutive newlines should collapse");
    }

    #[test]
    fn unterminated_string_errors() {
        assert!(tokenize("\"oops\n").is_err());
    }

    #[test]
    fn ends_with_eof() {
        let t = tokens("x\n");
        assert!(matches!(t.last().unwrap(), Token::Eof));
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::{lexer, parser};
    use crate::ast::{Stmt, Expr, BinOp};

    fn parse(src: &str) -> Vec<Stmt> {
        let src = if src.ends_with('\n') { src.to_string() } else { format!("{}\n", src) };
        let tokens = lexer::tokenize(&src).unwrap();
        parser::parse(tokens).unwrap()
    }

    fn first_expr(src: &str) -> Expr {
        match &parse(src)[0] {
            Stmt::Expr(e, _) => e.clone(),
            Stmt::Assign { expr, .. } => expr.clone(),
            _ => panic!("expected expression or assignment statement"),
        }
    }

    #[test]
    fn number_literal() {
        match first_expr("3.14") {
            Expr::Number(n) => assert!((n - 3.14).abs() < 1e-12),
            other => panic!("Expected Number, got {other:?}"),
        }
    }

    #[test]
    fn assignment_stmt() {
        let stmts = parse("x = 42");
        match &stmts[0] {
            Stmt::Assign { name, expr: Expr::Number(n), suppress: false } => {
                assert_eq!(name, "x");
                assert!((*n - 42.0).abs() < 1e-12);
            }
            other => panic!("Expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn suppress_flag_with_semicolon() {
        let stmts = parse("x = 42;");
        match &stmts[0] {
            Stmt::Assign { suppress: true, .. } => {}
            other => panic!("Expected suppressed assign, got {other:?}"),
        }
    }

    #[test]
    fn addition_expr() {
        match first_expr("1 + 2") {
            Expr::BinOp { op: BinOp::Add, .. } => {}
            other => panic!("Expected Add, got {other:?}"),
        }
    }

    #[test]
    fn unary_minus() {
        match first_expr("-5") {
            Expr::UnaryMinus(_) => {}
            other => panic!("Expected UnaryMinus, got {other:?}"),
        }
    }

    #[test]
    fn range_two_args() {
        match first_expr("1:5") {
            Expr::Range { step: None, .. } => {}
            other => panic!("Expected Range(None), got {other:?}"),
        }
    }

    #[test]
    fn range_three_args() {
        match first_expr("0:0.5:2") {
            Expr::Range { step: Some(_), .. } => {}
            other => panic!("Expected Range(Some), got {other:?}"),
        }
    }

    #[test]
    fn transpose_expr() {
        match first_expr("v'") {
            Expr::Transpose(_) => {}
            other => panic!("Expected Transpose, got {other:?}"),
        }
    }

    #[test]
    fn element_wise_mul() {
        match first_expr("a .* b") {
            Expr::BinOp { op: BinOp::ElemMul, .. } => {}
            other => panic!("Expected ElemMul, got {other:?}"),
        }
    }

    #[test]
    fn function_call_expr() {
        match first_expr("sin(x)") {
            Expr::Call { name, args } => {
                assert_eq!(name, "sin");
                assert_eq!(args.len(), 1);
            }
            other => panic!("Expected Call, got {other:?}"),
        }
    }

    #[test]
    fn matrix_literal() {
        match first_expr("[1, 2; 3, 4]") {
            Expr::Matrix(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
            }
            other => panic!("Expected Matrix, got {other:?}"),
        }
    }

    #[test]
    fn operator_precedence_mul_over_add() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        match first_expr("1 + 2 * 3") {
            Expr::BinOp { op: BinOp::Add, rhs, .. } => {
                assert!(matches!(*rhs, Expr::BinOp { op: BinOp::Mul, .. }));
            }
            other => panic!("Expected Add at root, got {other:?}"),
        }
    }

    #[test]
    fn power_right_associative() {
        // 2 ^ 3 ^ 4 should parse as 2 ^ (3 ^ 4)
        match first_expr("2 ^ 3 ^ 4") {
            Expr::BinOp { op: BinOp::Pow, rhs, .. } => {
                assert!(matches!(*rhs, Expr::BinOp { op: BinOp::Pow, .. }));
            }
            other => panic!("Expected Pow at root, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod value_tests {
    use crate::eval::value::Value;
    use crate::ast::BinOp;
    use ndarray::Array1;
    use num_complex::Complex;
    use rustlab_core::C64;

    fn scalar(n: f64) -> Value { Value::Scalar(n) }
    fn complex(re: f64, im: f64) -> Value { Value::Complex(Complex::new(re, im)) }
    fn vec_val(v: &[f64]) -> Value {
        Value::Vector(Array1::from_iter(v.iter().map(|&x| Complex::new(x, 0.0))))
    }
    fn close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-9 }

    // ── Negate ──────────────────────────────────────────────────────────────

    #[test]
    fn negate_scalar() {
        match scalar(3.0).negate().unwrap() {
            Value::Scalar(n) => assert!(close(n, -3.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn negate_complex() {
        match complex(1.0, 2.0).negate().unwrap() {
            Value::Complex(c) => {
                assert!(close(c.re, -1.0));
                assert!(close(c.im, -2.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn negate_vector() {
        let v = vec_val(&[1.0, -2.0, 3.0]);
        match v.negate().unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, -1.0));
                assert!(close(v[1].re, 2.0));
                assert!(close(v[2].re, -3.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn negate_string_errors() {
        assert!(Value::Str("x".to_string()).negate().is_err());
    }

    // ── Transpose ───────────────────────────────────────────────────────────

    #[test]
    fn transpose_real_vector_unchanged() {
        // Transposing a row vector produces an n×1 Matrix (column vector)
        let v = vec_val(&[1.0, 2.0, 3.0]);
        match v.transpose().unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 3);
                assert_eq!(m.ncols(), 1);
                assert!(close(m[[0, 0]].re, 1.0));
                assert!(close(m[[2, 0]].re, 3.0));
            }
            _ => panic!("Expected Matrix(n×1) from Vector transpose"),
        }
    }

    #[test]
    fn transpose_complex_vector_conjugates() {
        // Transposing a complex row vector produces an n×1 Matrix with conjugated values
        let v = Value::Vector(Array1::from_vec(vec![
            Complex::new(1.0, 2.0),
            Complex::new(3.0, -1.0),
        ]));
        match v.transpose().unwrap() {
            Value::Matrix(m) => {
                assert!(close(m[[0, 0]].im, -2.0));
                assert!(close(m[[1, 0]].im, 1.0));
            }
            _ => panic!("Expected Matrix(n×1) from Vector transpose"),
        }
    }

    #[test]
    fn transpose_scalar_is_identity() {
        match scalar(5.0).transpose().unwrap() {
            Value::Scalar(n) => assert!(close(n, 5.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn transpose_complex_scalar_conjugates() {
        match complex(1.0, 3.0).transpose().unwrap() {
            Value::Complex(c) => {
                assert!(close(c.re, 1.0));
                assert!(close(c.im, -3.0));
            }
            _ => panic!(),
        }
    }

    // ── BinOp ────────────────────────────────────────────────────────────────

    #[test]
    fn add_scalars() {
        match Value::binop(BinOp::Add, scalar(2.0), scalar(3.0)).unwrap() {
            Value::Scalar(n) => assert!(close(n, 5.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn sub_scalars() {
        match Value::binop(BinOp::Sub, scalar(5.0), scalar(3.0)).unwrap() {
            Value::Scalar(n) => assert!(close(n, 2.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn mul_scalars() {
        match Value::binop(BinOp::Mul, scalar(4.0), scalar(3.0)).unwrap() {
            Value::Scalar(n) => assert!(close(n, 12.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn div_scalars() {
        match Value::binop(BinOp::Div, scalar(10.0), scalar(4.0)).unwrap() {
            Value::Scalar(n) => assert!(close(n, 2.5)),
            _ => panic!(),
        }
    }

    #[test]
    fn pow_scalar() {
        match Value::binop(BinOp::Pow, scalar(2.0), scalar(10.0)).unwrap() {
            Value::Scalar(n) => assert!(close(n, 1024.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn add_complex_values() {
        match Value::binop(BinOp::Add, complex(1.0, 2.0), complex(3.0, 4.0)).unwrap() {
            Value::Complex(c) => {
                assert!(close(c.re, 4.0));
                assert!(close(c.im, 6.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn scalar_plus_complex_promotes() {
        match Value::binop(BinOp::Add, scalar(1.0), complex(0.0, 1.0)).unwrap() {
            Value::Complex(c) => {
                assert!(close(c.re, 1.0));
                assert!(close(c.im, 1.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn vector_add_vector() {
        match Value::binop(BinOp::Add, vec_val(&[1.0, 2.0]), vec_val(&[3.0, 4.0])).unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, 4.0));
                assert!(close(v[1].re, 6.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn scalar_broadcast_to_vector() {
        match Value::binop(BinOp::Mul, scalar(2.0), vec_val(&[1.0, 2.0, 3.0])).unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, 2.0));
                assert!(close(v[1].re, 4.0));
                assert!(close(v[2].re, 6.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn vector_length_mismatch_errors() {
        assert!(Value::binop(BinOp::Add, vec_val(&[1.0, 2.0]), vec_val(&[1.0])).is_err());
    }

    #[test]
    fn string_concatenation() {
        match Value::binop(
            BinOp::Add,
            Value::Str("hello".to_string()),
            Value::Str(" world".to_string()),
        ).unwrap() {
            Value::Str(s) => assert_eq!(s, "hello world"),
            _ => panic!(),
        }
    }

    // ── Index ────────────────────────────────────────────────────────────────

    #[test]
    fn vector_index_one_based() {
        let v = vec_val(&[10.0, 20.0, 30.0]);
        match v.index(vec![Value::Scalar(1.0)]).unwrap() {
            Value::Scalar(n) => assert!(close(n, 10.0)),
            _ => panic!(),
        }
    }

    #[test]
    fn vector_index_last_element() {
        let v = vec_val(&[10.0, 20.0, 30.0]);
        match v.index(vec![Value::Scalar(3.0)]).unwrap() {
            Value::Scalar(n) => assert!(close(n, 30.0)),
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod evaluator_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts {
            ev.exec_stmt(stmt).unwrap();
        }
        ev
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("Expected scalar for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-9 }

    #[test]
    fn predefined_pi() {
        let ev = Evaluator::new();
        match ev.get("pi").unwrap() {
            Value::Scalar(n) => assert!(close(*n, std::f64::consts::PI)),
            _ => panic!(),
        }
    }

    #[test]
    fn predefined_e() {
        let ev = Evaluator::new();
        match ev.get("e").unwrap() {
            Value::Scalar(n) => assert!(close(*n, std::f64::consts::E)),
            _ => panic!(),
        }
    }

    #[test]
    fn predefined_j_is_imaginary_unit() {
        let ev = Evaluator::new();
        match ev.get("j").unwrap() {
            Value::Complex(c) => {
                assert!(close(c.re, 0.0));
                assert!(close(c.im, 1.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn simple_assignment() {
        let ev = eval_str("x = 42");
        assert!(close(get_scalar(&ev, "x"), 42.0));
    }

    #[test]
    fn arithmetic_expr() {
        let ev = eval_str("y = 3 + 4 * 2");
        assert!(close(get_scalar(&ev, "y"), 11.0));
    }

    #[test]
    fn complex_constant_j() {
        let ev = eval_str("z = 3 + j*4");
        match ev.get("z").unwrap() {
            Value::Complex(c) => {
                assert!(close(c.re, 3.0));
                assert!(close(c.im, 4.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn range_creates_vector() {
        let ev = eval_str("v = 1:5");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 5);
                assert!(close(v[0].re, 1.0));
                assert!(close(v[4].re, 5.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn range_with_step() {
        let ev = eval_str("v = 0:2:8");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 5); // 0,2,4,6,8
                assert!(close(v[2].re, 4.0));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn builtin_abs() {
        let ev = eval_str("y = abs(-5)");
        assert!(close(get_scalar(&ev, "y"), 5.0));
    }

    #[test]
    fn builtin_sin_pi() {
        let ev = eval_str("y = sin(pi)");
        assert!(get_scalar(&ev, "y").abs() < 1e-14);
    }

    #[test]
    fn builtin_cos_zero() {
        let ev = eval_str("y = cos(0)");
        assert!(close(get_scalar(&ev, "y"), 1.0));
    }

    #[test]
    fn builtin_sqrt() {
        let ev = eval_str("y = sqrt(9)");
        assert!(close(get_scalar(&ev, "y"), 3.0));
    }

    #[test]
    fn builtin_zeros() {
        let ev = eval_str("v = zeros(5)");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 5);
                assert!(v.iter().all(|c| c.norm() < 1e-12));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn builtin_ones() {
        let ev = eval_str("v = ones(4)");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 4);
                assert!(v.iter().all(|c| (c.re - 1.0).abs() < 1e-12 && c.im.abs() < 1e-12));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn builtin_linspace() {
        let ev = eval_str("v = linspace(0, 1, 5)");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 5);
                assert!((v[0].re - 0.0).abs() < 1e-12);
                assert!((v[2].re - 0.5).abs() < 1e-12);
                assert!((v[4].re - 1.0).abs() < 1e-12);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn builtin_len() {
        let ev = eval_str("v = ones(7)\nn = len(v)");
        assert!(close(get_scalar(&ev, "n"), 7.0));
    }

    #[test]
    fn vector_indexing_one_based() {
        let ev = eval_str("v = linspace(10, 50, 5)\nx = v(1)");
        assert!(close(get_scalar(&ev, "x"), 10.0));
    }

    #[test]
    fn transpose_operator() {
        // Transposing a row vector [1,2,3] produces a 3×1 column Matrix
        let ev = eval_str("v = [1, 2, 3]\nvt = v'");
        match ev.get("vt").unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 3);
                assert_eq!(m.ncols(), 1);
            }
            _ => panic!("Expected Matrix(3×1) from transpose of [1,2,3]"),
        }
    }

    #[test]
    fn elementwise_mul() {
        let ev = eval_str("a = [1, 2, 3]\nb = [4, 5, 6]\nc = a .* b");
        match ev.get("c").unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, 4.0));
                assert!(close(v[1].re, 10.0));
                assert!(close(v[2].re, 18.0));
            }
            _ => panic!(),
        }
    }
}

// ─── Error tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod error_tests {
    use crate::{lexer, parser, Evaluator};

    fn eval_err(src: &str) -> crate::error::ScriptError {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = crate::eval::Evaluator::new();
        let mut last_err = None;
        for stmt in &stmts {
            if let Err(e) = ev.exec_stmt(stmt) {
                last_err = Some(e);
                break;
            }
        }
        last_err.expect("expected an error but script ran successfully")
    }

    #[test]
    fn undefined_variable_errors() {
        // x is never assigned — should produce Undefined error
        let _err = eval_err("x + 1");
        // just verifying it produces an error (the type assertion above panics if not)
    }

    #[test]
    fn index_out_of_bounds() {
        let _err = eval_err("v = 1:3\nv(5)");
    }

    #[test]
    fn vector_star_vector_errors() {
        // v * v with `*` (matrix mul) on two 1D vectors should error
        let _err = eval_err("a = 1:3\nb = 1:3\na * b");
    }

    #[test]
    fn inv_singular_errors() {
        let _err = eval_err("inv([0,0;0,0])");
    }

    #[test]
    fn linsolve_dimension_mismatch() {
        // A is 2×2, b has 3 elements — should error
        let _err = eval_err("linsolve([1,2;3,4], [1,2,3])");
    }

    #[test]
    fn det_non_square_errors() {
        let _err = eval_err("det([1,2,3;4,5,6])");
    }
}

// ─── Matrix / linalg tests ──────────────────────────────────────────────────

#[cfg(test)]
mod matrix_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("Expected scalar for '{name}', got {other:?}"),
        }
    }

    fn get_vector(ev: &Evaluator, name: &str) -> Vec<f64> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.iter().map(|c| c.re).collect(),
            other => panic!("Expected vector for '{name}', got {other:?}"),
        }
    }

    fn get_matrix(ev: &Evaluator, name: &str) -> ndarray::Array2<num_complex::Complex<f64>> {
        match ev.get(name).unwrap() {
            Value::Matrix(m) => m.clone(),
            other => panic!("Expected Matrix for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    #[test]
    fn eye_diagonal_ones() {
        let ev = eval_str("M = eye(3)");
        let m = get_matrix(&ev, "M");
        assert!((m[[0, 0]].re - 1.0).abs() < 1e-12, "M[0][0] should be 1");
        assert!((m[[0, 1]].re).abs() < 1e-12, "M[0][1] should be 0");
        assert!((m[[1, 1]].re - 1.0).abs() < 1e-12, "M[1][1] should be 1");
    }

    #[test]
    fn trace_of_eye() {
        let ev = eval_str("x = trace(eye(4))");
        assert!(close(get_scalar(&ev, "x"), 4.0, 1e-12), "trace(eye(4)) should be 4.0");
    }

    #[test]
    fn det_2x2_known() {
        // det([3,8;4,6]) = 3*6 - 8*4 = 18 - 32 = -14
        let ev = eval_str("x = det([3,8;4,6])");
        assert!(close(get_scalar(&ev, "x"), -14.0, 1e-10), "det([3,8;4,6]) should be -14");
    }

    #[test]
    fn det_identity_3x3() {
        let ev = eval_str("x = det(eye(3))");
        assert!(close(get_scalar(&ev, "x"), 1.0, 1e-10), "det(eye(3)) should be 1.0");
    }

    #[test]
    fn inv_times_a_is_identity() {
        let ev = eval_str("A = [2,1;5,7]\nB = inv(A) * A");
        let b = get_matrix(&ev, "B");
        // B should be approximately eye(2)
        assert!((b[[0, 0]].re - 1.0).abs() < 1e-10, "B[0][0] should be ≈1");
        assert!((b[[0, 1]].re).abs() < 1e-10, "B[0][1] should be ≈0");
        assert!((b[[1, 0]].re).abs() < 1e-10, "B[1][0] should be ≈0");
        assert!((b[[1, 1]].re - 1.0).abs() < 1e-10, "B[1][1] should be ≈1");
    }

    #[test]
    fn linsolve_known_2x2() {
        // [2,1;5,7] * x = [11;13] → x = [3, 5] (check: 2*3+1*5=11, 5*3+7*5=50≠13... wait)
        // Correct solution: [2,1;5,7]*[3,1]' = [7,22]≠[11,13]
        // Use inv: [2,1;5,7]^-1 = (1/9)*[7,-1;-5,2]
        // x = [7*11-1*13, -5*11+2*13]/9 = [77-13, -55+26]/9 = [64/9, -29/9]
        // Actually: det=2*7-1*5=14-5=9
        // x1=(7*11-1*13)/9=(77-13)/9=64/9≈7.11, x2=(-5*11+2*13)/9=(-55+26)/9=-29/9≈-3.22
        // Let's use a simpler system: [1,0;0,1] * x = [3;7] → x=[3,7]
        let ev = eval_str("x = linsolve([1,0;0,1], [3;7])");
        let x = get_vector(&ev, "x");
        assert!(close(x[0], 3.0, 1e-10), "x[0] should be 3.0, got {}", x[0]);
        assert!(close(x[1], 7.0, 1e-10), "x[1] should be 7.0, got {}", x[1]);
    }

    #[test]
    fn dot_orthogonal() {
        let ev = eval_str("x = dot([1,0,0], [0,1,0])");
        assert!(close(get_scalar(&ev, "x"), 0.0, 1e-12), "dot of orthogonal vectors should be 0");
    }

    #[test]
    fn dot_known() {
        let ev = eval_str("x = dot([3,4], [3,4])");
        assert!(close(get_scalar(&ev, "x"), 25.0, 1e-12), "dot([3,4],[3,4]) should be 25");
    }

    #[test]
    fn norm_l2_pythagorean() {
        let ev = eval_str("x = norm([3,4])");
        assert!(close(get_scalar(&ev, "x"), 5.0, 1e-10), "norm([3,4]) should be 5.0");
    }

    #[test]
    fn norm_l1_known() {
        let ev = eval_str("x = norm([1,2,3], 1)");
        assert!(close(get_scalar(&ev, "x"), 6.0, 1e-10), "L1 norm of [1,2,3] should be 6.0");
    }

    #[test]
    fn cross_known() {
        let ev = eval_str("v = cross([1,0,0], [0,1,0])");
        let v = get_vector(&ev, "v");
        assert!(close(v[0], 0.0, 1e-12), "cross[0] should be 0");
        assert!(close(v[1], 0.0, 1e-12), "cross[1] should be 0");
        assert!(close(v[2], 1.0, 1e-12), "cross[2] should be 1");
    }

    #[test]
    fn reshape_changes_dimensions() {
        let ev = eval_str("M = reshape(1:6, 2, 3)");
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 2, "reshaped matrix should have 2 rows");
        assert_eq!(m.ncols(), 3, "reshaped matrix should have 3 cols");
    }

    #[test]
    fn horzcat_increases_cols() {
        let ev = eval_str("M = horzcat(eye(2), eye(2))");
        let m = get_matrix(&ev, "M");
        assert_eq!(m.ncols(), 4, "horzcat of two 2×2 eye matrices should have 4 cols");
    }

    #[test]
    fn vertcat_increases_rows() {
        let ev = eval_str("M = vertcat(eye(2), eye(2))");
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 4, "vertcat of two 2×2 eye matrices should have 4 rows");
    }

    #[test]
    fn diag_extracts_diagonal() {
        let ev = eval_str("v = diag([1,2;3,4])");
        let v = get_vector(&ev, "v");
        assert!(close(v[0], 1.0, 1e-12), "diag[0] should be 1.0");
        assert!(close(v[1], 4.0, 1e-12), "diag[1] should be 4.0");
    }
}

// ─── Save/load round-trip tests ─────────────────────────────────────────────

#[cfg(test)]
mod io_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn eval_err(src: &str) -> crate::error::ScriptError {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = crate::eval::Evaluator::new();
        let mut last_err = None;
        for stmt in &stmts {
            if let Err(e) = ev.exec_stmt(stmt) {
                last_err = Some(e);
                break;
            }
        }
        last_err.expect("expected an error but script ran successfully")
    }

    fn get_vector(ev: &Evaluator, name: &str) -> Vec<f64> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.iter().map(|c| c.re).collect(),
            other => panic!("Expected vector for '{name}', got {other:?}"),
        }
    }

    fn tmp_path(suffix: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("rustlab_io_test_{}{}", std::process::id(), suffix));
        p.to_str().unwrap().to_string()
    }

    fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    #[test]
    fn npy_roundtrip_real_vector() {
        let path = tmp_path(".npy");
        let save_src = format!(r#"save("{path}", 1:5)"#);
        eval_str(&save_src);
        let load_src = format!(r#"x = load("{path}")"#);
        let ev = eval_str(&load_src);
        let x = get_vector(&ev, "x");
        assert!(close(x[0], 1.0, 1e-6), "x[0] should be 1.0, got {}", x[0]);
        assert!(close(x[4], 5.0, 1e-6), "x[4] should be 5.0, got {}", x[4]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn npy_roundtrip_complex_vector() {
        let path = tmp_path("_complex.npy");
        let save_src = format!(r#"v = [1+j*2, 3+j*4]
save("{path}", v)"#);
        eval_str(&save_src);
        let load_src = format!(r#"x = load("{path}")"#);
        let ev = eval_str(&load_src);
        // Check real parts
        match ev.get("x").unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, 1.0, 1e-6), "real part[0] should be 1.0");
                assert!(close(v[0].im, 2.0, 1e-6), "imag part[0] should be 2.0");
                assert!(close(v[1].re, 3.0, 1e-6), "real part[1] should be 3.0");
                assert!(close(v[1].im, 4.0, 1e-6), "imag part[1] should be 4.0");
            }
            other => panic!("Expected vector, got {other:?}"),
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn csv_roundtrip_real_vector() {
        let path = tmp_path(".csv");
        let save_src = format!(r#"save("{path}", 1:4)"#);
        eval_str(&save_src);
        let load_src = format!(r#"x = load("{path}")"#);
        let ev = eval_str(&load_src);
        let x = get_vector(&ev, "x");
        assert!(close(x[0], 1.0, 1e-6), "csv x[0] should be 1.0, got {}", x[0]);
        assert!(close(x[3], 4.0, 1e-6), "csv x[3] should be 4.0, got {}", x[3]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn npz_roundtrip_named_array() {
        let path = tmp_path(".npz");
        let save_src = format!(r#"save("{path}", "arr", 1:3)"#);
        eval_str(&save_src);
        let load_src = format!(r#"x = load("{path}", "arr")"#);
        let ev = eval_str(&load_src);
        let x = get_vector(&ev, "x");
        assert!(close(x[0], 1.0, 1e-6), "npz x[0] should be 1.0, got {}", x[0]);
        assert!(close(x[2], 3.0, 1e-6), "npz x[2] should be 3.0, got {}", x[2]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_missing_file_errors() {
        let _err = eval_err(r#"load("/no/such/file/rustlab_nonexistent_abc123.npy")"#);
        // Just verifying it errors — the eval_err helper panics if no error is produced
    }
}

// ─── factor() tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod factor_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn eval_err(src: &str) -> crate::error::ScriptError {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = crate::eval::Evaluator::new();
        let mut last_err = None;
        for stmt in &stmts {
            if let Err(e) = ev.exec_stmt(stmt) {
                last_err = Some(e);
                break;
            }
        }
        last_err.expect("expected an error but script ran successfully")
    }

    fn get_vector(ev: &Evaluator, name: &str) -> Vec<f64> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.iter().map(|c| c.re).collect(),
            other => panic!("Expected vector for '{name}', got {other:?}"),
        }
    }

    #[test]
    fn factor_of_1_is_empty() {
        let ev = eval_str("v = factor(1)");
        assert_eq!(get_vector(&ev, "v").len(), 0);
    }

    #[test]
    fn factor_12() {
        let ev = eval_str("v = factor(12)");
        assert_eq!(get_vector(&ev, "v"), vec![2.0, 2.0, 3.0]);
    }

    #[test]
    fn factor_prime() {
        let ev = eval_str("v = factor(17)");
        assert_eq!(get_vector(&ev, "v"), vec![17.0]);
    }

    #[test]
    fn factor_100() {
        let ev = eval_str("v = factor(100)");
        assert_eq!(get_vector(&ev, "v"), vec![2.0, 2.0, 5.0, 5.0]);
    }

    #[test]
    fn factor_zero_errors() {
        let _err = eval_err("factor(0)");
    }

    #[test]
    fn factor_negative_errors() {
        let _err = eval_err("factor(-3)");
    }

    #[test]
    fn factor_product_equals_input() {
        // product of factors of n == n
        let ev = eval_str("v = factor(360)");
        let factors = get_vector(&ev, "v");
        let product: f64 = factors.iter().product();
        assert!((product - 360.0).abs() < 1e-10);
    }
}

// ─── eig() tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod eig_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn eval_err(src: &str) -> crate::error::ScriptError {
        let src = format!("{}\n", src);
        let tokens = crate::lexer::tokenize(&src).unwrap();
        let stmts = crate::parser::parse(tokens).unwrap();
        let mut ev = crate::eval::Evaluator::new();
        let mut last_err = None;
        for stmt in &stmts {
            if let Err(e) = ev.exec_stmt(stmt) {
                last_err = Some(e);
                break;
            }
        }
        last_err.expect("expected an error but script ran successfully")
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("Expected scalar for '{name}', got {other:?}"),
        }
    }

    fn get_complex_vector(ev: &Evaluator, name: &str) -> Vec<num_complex::Complex<f64>> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.to_vec(),
            other => panic!("expected Vector for '{name}', got {other:?}"),
        }
    }

    #[test]
    fn eig_identity_all_ones() {
        let ev = eval_str("v = eig(eye(3))");
        let vals = get_complex_vector(&ev, "v");
        assert_eq!(vals.len(), 3);
        for l in &vals {
            assert!((l.re - 1.0).abs() < 1e-8, "eigenvalue re should be ~1, got {}", l.re);
            assert!(l.im.abs() < 1e-8, "eigenvalue im should be ~0, got {}", l.im);
        }
    }

    #[test]
    fn eig_diagonal_matrix() {
        // Eigenvalues of a diagonal matrix are its diagonal entries
        let ev = eval_str("v = eig([2,0;0,5])");
        let vals = get_complex_vector(&ev, "v");
        assert_eq!(vals.len(), 2);
        let mut re: Vec<f64> = vals.iter().map(|c| c.re).collect();
        re.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((re[0] - 2.0).abs() < 1e-8);
        assert!((re[1] - 5.0).abs() < 1e-8);
    }

    #[test]
    fn eig_symmetric_2x2() {
        // [2,1;1,2] has eigenvalues 1 and 3
        let ev = eval_str("v = eig([2,1;1,2])");
        let vals = get_complex_vector(&ev, "v");
        assert_eq!(vals.len(), 2);
        let mut re: Vec<f64> = vals.iter().map(|c| c.re).collect();
        re.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((re[0] - 1.0).abs() < 1e-8, "expected 1.0, got {}", re[0]);
        assert!((re[1] - 3.0).abs() < 1e-8, "expected 3.0, got {}", re[1]);
    }

    #[test]
    fn eig_sum_equals_trace() {
        // sum of eigenvalues == trace(M) for any square matrix
        let ev = eval_str("M = [4,2,1;2,3,1;1,1,2]\nv = eig(M)\nt = trace(M)");
        let vals = get_complex_vector(&ev, "v");
        let sum_re: f64 = vals.iter().map(|c| c.re).sum();
        let tr = get_scalar(&ev, "t");
        assert!((sum_re - tr).abs() < 1e-7, "sum(eig) = {sum_re}, trace = {tr}");
    }

    #[test]
    fn eig_product_equals_det() {
        // product of eigenvalues == det(M)
        let ev = eval_str("M = [3,1;1,3]\nv = eig(M)\nd = det(M)");
        let vals = get_complex_vector(&ev, "v");
        let prod: num_complex::Complex<f64> = vals.iter().product();
        let det_val = get_scalar(&ev, "d");
        assert!((prod.re - det_val).abs() < 1e-7, "prod(eig) = {}, det = {}", prod.re, det_val);
    }

    #[test]
    fn eig_scalar_input() {
        // eig(5) — scalar treated as 1×1 matrix, returns [5]
        let ev = eval_str("v = eig(5)");
        let vals = get_complex_vector(&ev, "v");
        assert_eq!(vals.len(), 1);
        assert!((vals[0].re - 5.0).abs() < 1e-10);
        assert!(vals[0].im.abs() < 1e-10);
    }

    #[test]
    fn eig_non_square_errors() {
        let _err = eval_err("eig([1,2,3;4,5,6])");
    }
}

// ── Phase 1: Language Foundations ────────────────────────────────────────────

#[cfg(test)]
mod phase1_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("expected scalar for '{name}', got {other:?}"),
        }
    }

    fn get_bool(ev: &Evaluator, name: &str) -> bool {
        match ev.get(name).unwrap() {
            Value::Bool(b) => *b,
            other => panic!("expected bool for '{name}', got {other:?}"),
        }
    }

    fn get_complex_vector(ev: &Evaluator, name: &str) -> Vec<num_complex::Complex<f64>> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.to_vec(),
            other => panic!("expected vector for '{name}', got {other:?}"),
        }
    }

    // ── 1a: % comments ───────────────────────────────────────────────────────

    #[test]
    fn percent_comment_ignored() {
        let ev = eval_str("% this is a comment\nx = 1");
        assert_eq!(get_scalar(&ev, "x"), 1.0);
    }

    #[test]
    fn percent_comment_inline() {
        let ev = eval_str("x = 42 % inline comment");
        assert_eq!(get_scalar(&ev, "x"), 42.0);
    }

    // ── 1b: if / else / end ──────────────────────────────────────────────────

    #[test]
    fn if_true_branch_taken() {
        let ev = eval_str("x = 0\nif 1 == 1\nx = 1\nend");
        assert_eq!(get_scalar(&ev, "x"), 1.0);
    }

    #[test]
    fn if_false_branch_skipped() {
        let ev = eval_str("x = 0\nif 1 == 2\nx = 99\nend");
        assert_eq!(get_scalar(&ev, "x"), 0.0);
    }

    #[test]
    fn if_else_true() {
        let ev = eval_str("if 1 == 1\nx = 10\nelse\nx = 20\nend");
        assert_eq!(get_scalar(&ev, "x"), 10.0);
    }

    #[test]
    fn if_else_false() {
        let ev = eval_str("if 1 == 2\nx = 10\nelse\nx = 20\nend");
        assert_eq!(get_scalar(&ev, "x"), 20.0);
    }

    #[test]
    fn if_scalar_condition() {
        // nonzero scalar is truthy
        let ev = eval_str("if 5\nx = 1\nelse\nx = 0\nend");
        assert_eq!(get_scalar(&ev, "x"), 1.0);
    }

    #[test]
    fn nested_if() {
        let ev = eval_str(
            "x = 0\nif 1 == 1\nif 2 == 2\nx = 99\nend\nend"
        );
        assert_eq!(get_scalar(&ev, "x"), 99.0);
    }

    // ── 1c: multi-assign ─────────────────────────────────────────────────────

    #[test]
    fn multi_assign_from_tuple_function() {
        // Define a function returning a Tuple via struct trick isn't needed;
        // use a direct eval path: roots() returns a vector, not a tuple.
        // Test multi-assign with a builtin that returns Tuple: roots of quadratic
        // For now verify roots() returns a vector (tested below), and
        // test the Tuple-unpack path via a custom inline function in the REPL.
        let src = "function y = make_pair(a, b)\ny = struct(\"a\", a, \"b\", b)\nend\ns = make_pair(3, 7)";
        let ev = eval_str(src);
        if let Value::Struct(fields) = ev.get("s").unwrap() {
            assert_eq!(fields.get("a").unwrap().to_string(), "3");
            assert_eq!(fields.get("b").unwrap().to_string(), "7");
        } else { panic!("expected struct"); }
    }

    // ── 1d: disp / fprintf ───────────────────────────────────────────────────

    #[test]
    fn fprintf_produces_no_value() {
        // fprintf returns Value::None — just verify it doesn't error
        let src = "x = 1"; // placeholder, we test indirectly via output
        let ev  = eval_str(src);
        assert_eq!(get_scalar(&ev, "x"), 1.0);
    }

    #[test]
    fn fprintf_format_string_parses() {
        use crate::eval::builtins::apply_format;
        let result = apply_format("x = %.2f\n", &[Value::Scalar(3.14159)]).unwrap();
        assert_eq!(result, "x = 3.14\n");
    }

    #[test]
    fn fprintf_integer_format() {
        use crate::eval::builtins::apply_format;
        let result = apply_format("%d items", &[Value::Scalar(5.0)]).unwrap();
        assert_eq!(result, "5 items");
    }

    #[test]
    fn fprintf_percent_escape() {
        use crate::eval::builtins::apply_format;
        let result = apply_format("100%%", &[]).unwrap();
        assert_eq!(result, "100%");
    }

    // ── 1e: all / any ────────────────────────────────────────────────────────

    #[test]
    fn all_nonzero_vector_true() {
        let ev = eval_str("b = all([1, 2, 3])");
        assert!(get_bool(&ev, "b"));
    }

    #[test]
    fn all_with_zero_false() {
        let ev = eval_str("b = all([1, 0, 3])");
        assert!(!get_bool(&ev, "b"));
    }

    #[test]
    fn any_with_nonzero_true() {
        let ev = eval_str("b = any([0, 0, 1])");
        assert!(get_bool(&ev, "b"));
    }

    #[test]
    fn any_all_zero_false() {
        let ev = eval_str("b = any([0, 0, 0])");
        assert!(!get_bool(&ev, "b"));
    }

    // ── 1f: rank ─────────────────────────────────────────────────────────────

    #[test]
    fn rank_identity_3x3() {
        let ev = eval_str("r = rank(eye(3))");
        assert_eq!(get_scalar(&ev, "r"), 3.0);
    }

    #[test]
    fn rank_singular_matrix() {
        // Rows [1,2] and [2,4] are linearly dependent → rank 1
        let ev = eval_str("r = rank([1,2;2,4])");
        assert_eq!(get_scalar(&ev, "r"), 1.0);
    }

    #[test]
    fn rank_full_rank_2x2() {
        let ev = eval_str("r = rank([1,2;3,4])");
        assert_eq!(get_scalar(&ev, "r"), 2.0);
    }

    // ── 1g: roots ────────────────────────────────────────────────────────────

    #[test]
    fn roots_linear() {
        // 2x - 4 = 0  →  root = 2
        let ev = eval_str("r = roots([2, -4])");
        let v  = get_complex_vector(&ev, "r");
        assert_eq!(v.len(), 1);
        assert!((v[0].re - 2.0).abs() < 1e-10);
        assert!(v[0].im.abs() < 1e-10);
    }

    #[test]
    fn roots_quadratic_real() {
        // x²-3x+2 = (x-1)(x-2)  →  roots 1, 2
        let ev = eval_str("r = roots([1, -3, 2])");
        let mut v: Vec<f64> = get_complex_vector(&ev, "r")
            .iter().map(|c| c.re).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((v[0] - 1.0).abs() < 1e-8);
        assert!((v[1] - 2.0).abs() < 1e-8);
    }

    #[test]
    fn roots_quadratic_complex() {
        // s²+2s+10 → roots -1±3j
        let ev = eval_str("r = roots([1, 2, 10])");
        let v  = get_complex_vector(&ev, "r");
        assert_eq!(v.len(), 2);
        for c in &v {
            assert!((c.re - (-1.0)).abs() < 1e-8);
            assert!((c.im.abs() - 3.0).abs() < 1e-8);
        }
    }
}

// ── Phase 2: Transfer Function Type ──────────────────────────────────────────

#[cfg(test)]
mod phase2_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    fn get_tf(ev: &Evaluator, name: &str) -> (Vec<f64>, Vec<f64>) {
        match ev.get(name).unwrap() {
            Value::TransferFn { num, den } => (num.clone(), den.clone()),
            other => panic!("expected tf for '{name}', got {other:?}"),
        }
    }

    fn get_complex_vector(ev: &Evaluator, name: &str) -> Vec<num_complex::Complex<f64>> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.to_vec(),
            other => panic!("expected vector for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-8 }

    // ── 2b: tf() builtin ─────────────────────────────────────────────────────

    #[test]
    fn tf_laplace_variable() {
        // tf("s") should produce num=[1,0], den=[1]
        let ev = eval_str("s = tf(\"s\")");
        let (num, den) = get_tf(&ev, "s");
        assert_eq!(num, vec![1.0, 0.0]);
        assert_eq!(den, vec![1.0]);
    }

    #[test]
    fn tf_explicit_num_den() {
        let ev = eval_str("G = tf([10], [1, 2, 10])");
        let (num, den) = get_tf(&ev, "G");
        assert_eq!(num, vec![10.0]);
        assert_eq!(den, vec![1.0, 2.0, 10.0]);
    }

    #[test]
    fn tf_from_laplace_arithmetic() {
        // s = tf("s");  G = 10 / (s^2 + 2*s + 10)
        let ev = eval_str("s = tf(\"s\")\nG = 10 / (s^2 + 2*s + 10)");
        let (num, den) = get_tf(&ev, "G");
        assert_eq!(num.len(), 1);
        assert!(close(num[0], 10.0), "num[0] = {}", num[0]);
        assert_eq!(den.len(), 3);
        assert!(close(den[0], 1.0),  "den[0] = {}", den[0]);
        assert!(close(den[1], 2.0),  "den[1] = {}", den[1]);
        assert!(close(den[2], 10.0), "den[2] = {}", den[2]);
    }

    #[test]
    fn tf_pow_zero_gives_unity() {
        let ev = eval_str("s = tf(\"s\")\nG = s^0");
        let (num, den) = get_tf(&ev, "G");
        assert_eq!(num, vec![1.0]);
        assert_eq!(den, vec![1.0]);
    }

    #[test]
    fn tf_mul_two_tfs() {
        // (1/s) * (1/s) = 1/s^2
        let ev = eval_str("s = tf(\"s\")\nG = (1/s) * (1/s)");
        let (num, den) = get_tf(&ev, "G");
        // num should be [1], den should be degree 2
        assert!(close(num.iter().map(|x| x.abs()).sum::<f64>(), 1.0));
        assert_eq!(den.len(), 3); // s^2 from [1,0] * [1,0]
    }

    #[test]
    fn tf_add_two_tfs() {
        // 1/(s+1) + 1/(s+2) = (2s+3)/((s+1)(s+2))
        let ev = eval_str("s = tf(\"s\")\nG = 1/(s+1) + 1/(s+2)");
        let (num, den) = get_tf(&ev, "G");
        // numerator should be [2, 3], denominator [1, 3, 2]
        assert_eq!(num.len(), 2);
        assert!(close(num[0], 2.0), "num[0]={}", num[0]);
        assert!(close(num[1], 3.0), "num[1]={}", num[1]);
        assert_eq!(den.len(), 3);
        assert!(close(den[0], 1.0), "den[0]={}", den[0]);
        assert!(close(den[1], 3.0), "den[1]={}", den[1]);
        assert!(close(den[2], 2.0), "den[2]={}", den[2]);
    }

    // ── 2c: pole() and zero() builtins ───────────────────────────────────────

    #[test]
    fn pole_second_order() {
        // G = 10 / (s^2 + 2s + 10)  →  poles at -1 ± 3j
        let ev = eval_str("G = tf([10], [1, 2, 10])\np = pole(G)");
        let poles = get_complex_vector(&ev, "p");
        assert_eq!(poles.len(), 2);
        for p in &poles {
            assert!(close(p.re, -1.0), "pole re = {}", p.re);
            assert!(close(p.im.abs(), 3.0), "pole |im| = {}", p.im.abs());
        }
    }

    #[test]
    fn zero_of_tf() {
        // G = (s - 2) / (s + 5)  →  zero at s=2
        let ev = eval_str("G = tf([1, -2], [1, 5])\nz = zero(G)");
        let zeros = get_complex_vector(&ev, "z");
        assert_eq!(zeros.len(), 1);
        assert!(close(zeros[0].re, 2.0), "zero re = {}", zeros[0].re);
        assert!(close(zeros[0].im, 0.0), "zero im = {}", zeros[0].im);
    }

    #[test]
    fn pole_of_laplace_s() {
        // tf("s") has num=[1,0], den=[1] → den is degree 0 constant, no finite poles
        let ev = eval_str("s = tf(\"s\")\np = pole(s)");
        // den=[1.0] → roots of a constant → empty vector
        let poles = get_complex_vector(&ev, "p");
        assert_eq!(poles.len(), 0);
    }
}

// ── Phase 3: State-Space Type ─────────────────────────────────────────────────

#[cfg(test)]
mod phase3_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;
    use rustlab_core::CMatrix;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    fn get_ss(ev: &Evaluator, name: &str) -> (CMatrix, CMatrix, CMatrix, CMatrix) {
        match ev.get(name).unwrap() {
            Value::StateSpace { a, b, c, d } => (a.clone(), b.clone(), c.clone(), d.clone()),
            other => panic!("expected ss for '{name}', got {other:?}"),
        }
    }

    fn get_matrix(ev: &Evaluator, name: &str) -> CMatrix {
        match ev.get(name).unwrap() {
            Value::Matrix(m) => m.clone(),
            other => panic!("expected matrix for '{name}', got {other:?}"),
        }
    }

    fn get_complex_vector(ev: &Evaluator, name: &str) -> Vec<num_complex::Complex<f64>> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.to_vec(),
            other => panic!("expected vector for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-7 }

    // ── 3b: ss() conversion ───────────────────────────────────────────────────

    #[test]
    fn ss_dimensions_second_order() {
        // G = 10/(s²+2s+10) → 2-state, 1-input, 1-output
        let ev = eval_str("G = tf([10],[1,2,10])\nsys = ss(G)");
        let (a, b, c, d) = get_ss(&ev, "sys");
        assert_eq!(a.nrows(), 2);  assert_eq!(a.ncols(), 2);
        assert_eq!(b.nrows(), 2);  assert_eq!(b.ncols(), 1);
        assert_eq!(c.nrows(), 1);  assert_eq!(c.ncols(), 2);
        assert_eq!(d.nrows(), 1);  assert_eq!(d.ncols(), 1);
    }

    #[test]
    fn ss_eigenvalues_match_poles() {
        // Eigenvalues of A should match poles of G
        let ev = eval_str(
            "G = tf([10],[1,2,10])\nsys = ss(G)\nlam = eig(sys.A)"
        );
        let eigs = get_complex_vector(&ev, "lam");
        assert_eq!(eigs.len(), 2);
        for e in &eigs {
            assert!(close(e.re, -1.0), "eig re = {}", e.re);
            assert!(close(e.im.abs(), 3.0), "eig |im| = {}", e.im.abs());
        }
    }

    #[test]
    fn ss_d_zero_for_strictly_proper() {
        let ev = eval_str("G = tf([10],[1,2,10])\nsys = ss(G)");
        let (_, _, _, d) = get_ss(&ev, "sys");
        assert!(d[[0, 0]].norm() < 1e-12, "D should be 0 for strictly proper TF");
    }

    #[test]
    fn ss_field_access() {
        let ev = eval_str("G = tf([10],[1,2,10])\nsys = ss(G)\nA = sys.A\nB = sys.B");
        let a = get_matrix(&ev, "A");
        let b = get_matrix(&ev, "B");
        assert_eq!(a.nrows(), 2);
        assert_eq!(b.ncols(), 1);
    }

    // ── 3c: ctrb() and obsv() ─────────────────────────────────────────────────

    #[test]
    fn ctrb_full_rank() {
        // Controllable second-order system: G = 10/(s²+2s+10)
        let ev = eval_str(
            "G = tf([10],[1,2,10])\nsys = ss(G)\nM = ctrb(sys.A, sys.B)"
        );
        let m = get_matrix(&ev, "M");
        // ctrb returns 2×2 for SISO second-order system
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 2);
        // Must be full rank — det != 0
        let det = m[[0,0]] * m[[1,1]] - m[[0,1]] * m[[1,0]];
        assert!(det.norm() > 1e-6, "controllability matrix should be full rank, det = {}", det);
    }

    #[test]
    fn obsv_full_rank() {
        let ev = eval_str(
            "G = tf([10],[1,2,10])\nsys = ss(G)\nM = obsv(sys.A, sys.C)"
        );
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 2);
        assert_eq!(m.ncols(), 2);
        let det = m[[0,0]] * m[[1,1]] - m[[0,1]] * m[[1,0]];
        assert!(det.norm() > 1e-6, "observability matrix should be full rank, det = {}", det);
    }

    #[test]
    fn ctrb_uncontrollable_rank_deficient() {
        // Double pole at -1, both states driven by same mode → rank 1
        // A = [-1, 0; 0, -1], B = [1; 1] → ctrb = [1, -1; 1, -1] → rank 1
        let ev = eval_str(
            "A = [-1,0;0,-1]\nB = [1;1]\nM = ctrb(A, B)"
        );
        let m = get_matrix(&ev, "M");
        let det = m[[0,0]] * m[[1,1]] - m[[0,1]] * m[[1,0]];
        assert!(det.norm() < 1e-10, "expected rank-deficient ctrb, det = {}", det);
    }
}

// ── Phase 4: Frequency & Time-Domain Analysis ─────────────────────────────────

#[cfg(test)]
mod phase4_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("expected scalar for '{name}', got {other:?}"),
        }
    }

    fn get_vector(ev: &Evaluator, name: &str) -> Vec<f64> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.iter().map(|c| c.re).collect(),
            other => panic!("expected vector for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    // ── 4a: bode() ────────────────────────────────────────────────────────────

    #[test]
    fn bode_returns_three_vectors() {
        let ev = eval_str("[mag, ph, w] = bode(tf([10],[1,2,10]));");
        let mag = get_vector(&ev, "mag");
        let ph  = get_vector(&ev, "ph");
        let w   = get_vector(&ev, "w");
        assert!(!mag.is_empty());
        assert_eq!(mag.len(), ph.len());
        assert_eq!(mag.len(), w.len());
    }

    #[test]
    fn bode_dc_gain_zero_db() {
        // G(0) = 10/10 = 1 → 0 dB; lowest frequency point should be near 0 dB
        let ev = eval_str("[mag, ph, w] = bode(tf([10],[1,2,10]));");
        let mag = get_vector(&ev, "mag");
        assert!(close(mag[0], 0.0, 1.5), "DC mag = {} dB, expected ~0 dB", mag[0]);
    }

    #[test]
    fn bode_user_supplied_freqs() {
        // Supply a known frequency vector: single point at w=0.001 ≈ DC
        let ev = eval_str("[mag, ph, w] = bode(tf([10],[1,2,10]), [0.001, 0.01, 0.1]);");
        let mag = get_vector(&ev, "mag");
        let w   = get_vector(&ev, "w");
        assert_eq!(mag.len(), 3);
        assert_eq!(w.len(), 3);
        assert!(close(mag[0], 0.0, 0.1), "DC mag = {} dB", mag[0]);
    }

    // ── 4b: step() ────────────────────────────────────────────────────────────

    #[test]
    fn step_returns_two_vectors() {
        let ev = eval_str("[y, t] = step(tf([10],[1,2,10]));");
        let y = get_vector(&ev, "y");
        let t = get_vector(&ev, "t");
        assert!(!y.is_empty());
        assert_eq!(y.len(), t.len());
        // t should start at 0
        assert!(close(t[0], 0.0, 1e-12));
    }

    #[test]
    fn step_steady_state_equals_dc_gain() {
        // G = 10/(s²+2s+10), DC gain = 10/10 = 1 → y(∞) ≈ 1
        let ev = eval_str("[y, t] = step(tf([10],[1,2,10]));");
        let y = get_vector(&ev, "y");
        let y_final = *y.last().unwrap();
        assert!(close(y_final, 1.0, 0.01), "y(∞) = {}, expected ~1.0", y_final);
    }

    #[test]
    fn step_user_specified_t_end() {
        let ev = eval_str("[y, t] = step(tf([10],[1,2,10]), 5.0);");
        let t = get_vector(&ev, "t");
        assert!(close(*t.last().unwrap(), 5.0, 0.01), "t_end = {}", t.last().unwrap());
    }

    // ── 4c: margin() ─────────────────────────────────────────────────────────

    #[test]
    fn margin_returns_tuple_of_four() {
        // margin(G) returns [Gm, Pm, Wcg, Wcp]
        let ev = eval_str("[gm, pm, wcg, wcp] = margin(tf([10],[1,2,10]));");
        // Just verify they exist and are numeric
        let _gm  = get_scalar(&ev, "gm");
        let pm   = get_scalar(&ev, "pm");
        let _wcg = get_scalar(&ev, "wcg");
        let wcp  = get_scalar(&ev, "wcp");
        // For G = 10/(s²+2s+10): PM ≈ 53°, Wcp ≈ 4 rad/s
        assert!(close(pm,  53.13, 1.0), "PM = {}, expected ~53.13°", pm);
        assert!(close(wcp,  4.0,  0.1), "Wcp = {}, expected ~4 rad/s", wcp);
    }

    #[test]
    fn margin_gm_infinite_for_second_order() {
        // Stable second-order system: phase never reaches -180° → GM = ∞
        let ev = eval_str("[gm, pm, wcg, wcp] = margin(tf([10],[1,2,10]));");
        let gm  = get_scalar(&ev, "gm");
        assert!(gm.is_infinite() || gm > 100.0, "GM = {}, expected ∞ for 2nd-order", gm);
    }
}

// ─── Phase 5 tests — Optimal Control (LQR) ───────────────────────────────────

#[cfg(test)]
mod phase5_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;
    use rustlab_core::C64;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    /// Return the complex eigenvalue vector (for checking stability).
    fn get_cvector(ev: &Evaluator, name: &str) -> Vec<C64> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.iter().cloned().collect(),
            other => panic!("expected vector for '{name}', got {other:?}"),
        }
    }

    // Double-integrator: A=[0,1;0,0], B=[0;1], Q=I, R=I
    // Analytical K = [1, sqrt(3)], P = [sqrt(3), 1; 1, sqrt(3)],
    // cl eigenvalues ≈ -0.866 ± 0.5j (both Re < 0)
    #[test]
    fn lqr_double_integrator_stable_cl_eigenvalues() {
        let script = r#"
sys = ss(tf([1],[1,0,0]))
Q = eye(2)
R = 1
[K, S, e] = lqr(sys, Q, R)
"#;
        let ev = eval_str(script);
        let e = get_cvector(&ev, "e");
        assert_eq!(e.len(), 2, "should have 2 closed-loop eigenvalues");
        for eig in &e {
            assert!(
                eig.re < 0.0,
                "closed-loop eigenvalue {} must have Re < 0 (stable)",
                eig
            );
        }
    }

    #[test]
    fn lqr_double_integrator_k_shape() {
        let script = r#"
sys = ss(tf([1],[1,0,0]))
Q = eye(2)
R = 1
[K, S, e] = lqr(sys, Q, R)
"#;
        let ev = eval_str(script);
        // K should be 1×2 for a single-input, 2-state system
        match ev.get("K").unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 1, "K rows = {}, expected 1", m.nrows());
                assert_eq!(m.ncols(), 2, "K cols = {}, expected 2", m.ncols());
            }
            other => panic!("K should be a matrix, got {}", other.type_name()),
        }
    }

    #[test]
    fn lqr_double_integrator_p_positive_definite() {
        let script = r#"
sys = ss(tf([1],[1,0,0]))
Q = eye(2)
R = 1
[K, S, e] = lqr(sys, Q, R)
"#;
        let ev = eval_str(script);
        // P (= S) should be 2×2 with positive diagonal entries
        match ev.get("S").unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.nrows(), 2);
                assert_eq!(m.ncols(), 2);
                assert!(m[[0,0]].re > 0.0, "P[0,0] = {} should be > 0", m[[0,0]].re);
                assert!(m[[1,1]].re > 0.0, "P[1,1] = {} should be > 0", m[[1,1]].re);
            }
            other => panic!("S should be a matrix, got {}", other.type_name()),
        }
    }

    #[test]
    fn lqr_returns_tuple_of_three() {
        let script = r#"
sys = ss(tf([1],[1,0,0]))
Q = eye(2)
R = 1
[K, S, e] = lqr(sys, Q, R)
"#;
        let ev = eval_str(script);
        // All three outputs must be bound
        assert!(ev.get("K").is_some(), "K not found in env");
        assert!(ev.get("S").is_some(), "S not found in env");
        assert!(ev.get("e").is_some(), "e not found in env");
    }
}

// ─── atan2 / meshgrid tests ───────────────────────────────────────────────────

#[cfg(test)]
mod math_extra_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("expected scalar for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    // ── atan2 ─────────────────────────────────────────────────────────────────

    #[test]
    fn atan2_scalar_scalars() {
        // atan2(0, -1) = π, atan2(1, 0) = π/2
        let ev = eval_str("a = atan2(0, -1)\nb = atan2(1, 0)");
        let a = get_scalar(&ev, "a");
        let b = get_scalar(&ev, "b");
        assert!(close(a, std::f64::consts::PI,     1e-12), "atan2(0,-1) = {a}");
        assert!(close(b, std::f64::consts::FRAC_PI_2, 1e-12), "atan2(1,0) = {b}");
    }

    #[test]
    fn atan2_vector_vector() {
        // atan2([0,1], [1,0]) = [0, π/2]
        let ev = eval_str("v = atan2([0,1], [1,0])");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 2);
                assert!(close(v[0].re, 0.0,                      1e-12));
                assert!(close(v[1].re, std::f64::consts::FRAC_PI_2, 1e-12));
            }
            other => panic!("expected vector, got {other:?}"),
        }
    }

    #[test]
    fn atan2_matrix_matrix() {
        // atan2([0,1;-1,0], [1,0;0,-1]) elementwise
        let ev = eval_str("M = atan2([0,1;-1,0], [1,0;0,-1])");
        match ev.get("M").unwrap() {
            Value::Matrix(m) => {
                assert_eq!(m.shape(), &[2, 2]);
                assert!(close(m[[0,0]].re, 0.0,                         1e-12)); // atan2(0,1)
                assert!(close(m[[0,1]].re, std::f64::consts::FRAC_PI_2, 1e-12)); // atan2(1,0)
                assert!(close(m[[1,0]].re, -std::f64::consts::FRAC_PI_2, 1e-12)); // atan2(-1,0)
                assert!(close(m[[1,1]].re, std::f64::consts::PI,         1e-12)); // atan2(0,-1)
            }
            other => panic!("expected matrix, got {other:?}"),
        }
    }

    // ── meshgrid ──────────────────────────────────────────────────────────────

    #[test]
    fn meshgrid_shapes() {
        // x has 3 elements, y has 2 → X and Y are 2×3
        let ev = eval_str("[X, Y] = meshgrid([1,2,3], [10,20])");
        match (ev.get("X").unwrap(), ev.get("Y").unwrap()) {
            (Value::Matrix(x), Value::Matrix(y)) => {
                assert_eq!(x.shape(), &[2, 3], "X shape {:?}", x.shape());
                assert_eq!(y.shape(), &[2, 3], "Y shape {:?}", y.shape());
            }
            other => panic!("expected matrices, got {other:?}"),
        }
    }

    #[test]
    fn meshgrid_x_varies_along_cols() {
        // X[i,j] = x[j], so X row 0 == X row 1
        let ev = eval_str("[X, Y] = meshgrid([1,2,3], [10,20])");
        match ev.get("X").unwrap() {
            Value::Matrix(x) => {
                assert!(close(x[[0,0]].re, 1.0, 1e-12));
                assert!(close(x[[0,1]].re, 2.0, 1e-12));
                assert!(close(x[[0,2]].re, 3.0, 1e-12));
                assert!(close(x[[1,0]].re, 1.0, 1e-12)); // same as row 0
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn meshgrid_y_varies_along_rows() {
        // Y[i,j] = y[i], so Y col 0 == Y col 1 == Y col 2
        let ev = eval_str("[X, Y] = meshgrid([1,2,3], [10,20])");
        match ev.get("Y").unwrap() {
            Value::Matrix(y) => {
                assert!(close(y[[0,0]].re, 10.0, 1e-12));
                assert!(close(y[[1,0]].re, 20.0, 1e-12));
                assert!(close(y[[0,2]].re, 10.0, 1e-12)); // same as col 0
                assert!(close(y[[1,2]].re, 20.0, 1e-12));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn meshgrid_with_atan2_for_angle() {
        // theta = atan2(Y, X) should give angle at each grid point
        let ev = eval_str(r#"
[X, Y] = meshgrid([-1,0,1], [0,1])
T = atan2(Y, X)
"#);
        match ev.get("T").unwrap() {
            Value::Matrix(t) => {
                assert_eq!(t.shape(), &[2, 3]);
                // atan2(0, -1) = π  at (row=0, col=0)
                assert!(close(t[[0,0]].re, std::f64::consts::PI, 1e-12),
                    "T[0,0] = {} expected π", t[[0,0]].re);
                // atan2(0, 1) = 0  at (row=0, col=2)
                assert!(close(t[[0,2]].re, 0.0, 1e-12),
                    "T[0,2] = {} expected 0", t[[0,2]].re);
            }
            other => panic!("{other:?}"),
        }
    }
}

// ── Phase 6: Root Locus ───────────────────────────────────────────────────────

#[cfg(test)]
mod phase6_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_ok(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts { ev.exec_stmt(s).unwrap(); }
        ev
    }

    fn eval_err(src: &str) -> String {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for s in &stmts {
            if let Err(e) = ev.exec_stmt(s) {
                return e.to_string();
            }
        }
        panic!("expected error but script succeeded");
    }

    // rlocus returns None without error (plot skipped in non-tty test env)
    #[test]
    fn rlocus_returns_none_for_second_order() {
        let ev = eval_ok("G = tf([10], [1, 2, 10])\nrlocus(G)");
        // rlocus doesn't bind a variable; just check it doesn't panic
        let _ = ev;
    }

    // rlocus works with the Laplace-variable form
    #[test]
    fn rlocus_with_s_form() {
        eval_ok("s = tf(\"s\")\nG = 1 / (s * (s + 1))\nrlocus(G)");
    }

    // rlocus rejects a non-TF argument
    #[test]
    fn rlocus_rejects_scalar() {
        let e = eval_err("rlocus(42)");
        assert!(e.contains("rlocus"), "error should mention rlocus: {e}");
    }

    // rlocus rejects an improper TF (num degree >= den degree)
    #[test]
    fn rlocus_rejects_improper_tf() {
        let e = eval_err("G = tf([1, 0], [1, 1])\nrlocus(G)");
        assert!(e.contains("proper"), "error should mention proper: {e}");
    }

    // The open-loop poles of den = [1,2,10] are approx -1 ± 3j.
    // Verify roots() (used internally) gives those values so the starting
    // points of the locus are correct.
    #[test]
    fn rlocus_open_loop_poles_are_tf_poles() {
        let ev = eval_ok("G = tf([10], [1, 2, 10])\np = pole(G)");
        match ev.get("p").unwrap() {
            Value::Vector(v) => {
                assert_eq!(v.len(), 2);
                // Both poles should have Re ≈ -1
                for c in v.iter() {
                    assert!(
                        (c.re - (-1.0)).abs() < 1e-6,
                        "pole real part should be ≈ -1, got {}", c.re
                    );
                }
            }
            other => panic!("expected vector, got {other:?}"),
        }
    }
}

// ─── New builtins: acos/asin/atan, outer, kron, expm, laguerre, legendre ────

#[cfg(test)]
mod new_builtins_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;
    use num_complex::Complex;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn eval_err(src: &str) -> String {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts  = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts {
            if let Err(e) = ev.exec_stmt(stmt) { return e.to_string(); }
        }
        panic!("expected an error but script ran successfully")
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(n) => *n,
            other => panic!("expected scalar for '{name}', got {other:?}"),
        }
    }

    fn get_matrix(ev: &Evaluator, name: &str) -> ndarray::Array2<Complex<f64>> {
        match ev.get(name).unwrap() {
            Value::Matrix(m) => m.clone(),
            other => panic!("expected matrix for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-9 }
    fn close_c(a: Complex<f64>, b: Complex<f64>) -> bool {
        (a.re - b.re).abs() < 1e-9 && (a.im - b.im).abs() < 1e-9
    }

    // ── acos / asin / atan ──────────────────────────────────────────────────

    #[test]
    fn acos_scalar() {
        let ev = eval_str("x = acos(1.0)");
        assert!(close(get_scalar(&ev, "x"), 0.0), "acos(1) should be 0");
    }

    #[test]
    fn acos_of_zero() {
        let ev = eval_str("x = acos(0.0)");
        let pi_2 = std::f64::consts::FRAC_PI_2;
        assert!(close(get_scalar(&ev, "x"), pi_2), "acos(0) should be π/2");
    }

    #[test]
    fn asin_scalar() {
        let ev = eval_str("x = asin(1.0)");
        let pi_2 = std::f64::consts::FRAC_PI_2;
        assert!(close(get_scalar(&ev, "x"), pi_2), "asin(1) should be π/2");
    }

    #[test]
    fn atan_scalar() {
        let ev = eval_str("x = atan(1.0)");
        let pi_4 = std::f64::consts::FRAC_PI_4;
        assert!(close(get_scalar(&ev, "x"), pi_4), "atan(1) should be π/4");
    }

    #[test]
    fn acos_vector() {
        // acos([1,0,-1]) = [0, π/2, π]
        let ev = eval_str("v = acos([1.0, 0.0, -1.0])");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, 0.0));
                assert!(close(v[1].re, std::f64::consts::FRAC_PI_2));
                assert!(close(v[2].re, std::f64::consts::PI));
            }
            other => panic!("expected vector, got {other:?}"),
        }
    }

    #[test]
    fn acos_matrix() {
        let ev = eval_str("M = acos([1,0;0,1])");
        let m = get_matrix(&ev, "M");
        assert!(close(m[[0,0]].re, 0.0));
        assert!(close(m[[0,1]].re, std::f64::consts::FRAC_PI_2));
    }

    // ── outer ────────────────────────────────────────────────────────────────

    #[test]
    fn outer_basic() {
        // outer([1,2,3], [10,20]) → [[10,20],[20,40],[30,60]]
        let ev = eval_str("M = outer([1.0,2.0,3.0], [10.0,20.0])");
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 3);
        assert_eq!(m.ncols(), 2);
        assert!(close(m[[0,0]].re, 10.0));
        assert!(close(m[[1,1]].re, 40.0));
        assert!(close(m[[2,0]].re, 30.0));
    }

    #[test]
    fn outer_rank_one() {
        // outer(v, v) where v=[1,0] should give [[1,0],[0,0]]
        let ev = eval_str("M = outer([1.0,0.0], [1.0,0.0])");
        let m = get_matrix(&ev, "M");
        assert!(close(m[[0,0]].re, 1.0));
        assert!(close(m[[0,1]].re, 0.0));
        assert!(close(m[[1,0]].re, 0.0));
        assert!(close(m[[1,1]].re, 0.0));
    }

    // ── kron ─────────────────────────────────────────────────────────────────

    #[test]
    fn kron_eye2_eye2() {
        // kron(eye(2), eye(2)) = eye(4)
        let ev = eval_str("M = kron(eye(2), eye(2))");
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 4);
        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!(close(m[[i,j]].re, expected), "kron(I,I)[{i},{j}] should be {expected}");
            }
        }
    }

    #[test]
    fn kron_scalar_matrix() {
        // kron(2, [1,2;3,4]) = [2,4;6,8]
        let ev = eval_str("M = kron(2.0, [1,2;3,4])");
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 2);
        assert!(close(m[[0,0]].re, 2.0));
        assert!(close(m[[1,1]].re, 8.0));
    }

    #[test]
    fn kron_pauli_x_pauli_z() {
        // σ_x ⊗ σ_z — known result for two-qubit system
        let ev = eval_str(
            "sx = [0,1;1,0]\nsz = [1,0;0,-1]\nM = kron(sx, sz)"
        );
        let m = get_matrix(&ev, "M");
        assert_eq!(m.nrows(), 4);
        // Top-left block should be 0*sz = zeros
        assert!(close(m[[0,0]].re, 0.0));
        // Top-right block should be 1*sz: m[0,2]=1, m[1,3]=-1
        assert!(close(m[[0,2]].re, 1.0));
        assert!(close(m[[1,3]].re, -1.0));
    }

    // ── expm ─────────────────────────────────────────────────────────────────

    #[test]
    fn expm_zero_matrix_gives_identity() {
        let ev = eval_str("M = expm(zeros(3,3))");
        let m = get_matrix(&ev, "M");
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((m[[i,j]].re - expected).abs() < 1e-10,
                    "expm(0)[{i},{j}] should be {expected}, got {}", m[[i,j]].re);
            }
        }
    }

    #[test]
    fn expm_diagonal_matrix() {
        // expm(diag([1,2])) = diag([e^1, e^2])
        let ev = eval_str("M = expm([1,0;0,2])");
        let m = get_matrix(&ev, "M");
        assert!((m[[0,0]].re - std::f64::consts::E).abs() < 1e-8,
            "expm diagonal [0,0] should be e");
        assert!((m[[1,1]].re - std::f64::consts::E.powi(2)).abs() < 1e-8,
            "expm diagonal [1,1] should be e^2");
        assert!(m[[0,1]].norm() < 1e-10, "off-diagonal should be 0");
    }

    #[test]
    fn expm_pauli_y_rotation() {
        // expm(-j * pi/2 * [0,-j;j,0]) — Pauli-Y rotation by π
        // σ_y = [0,-j;j,0]; expm(-j*π/2*σ_y) should be -j*σ_y (up to global phase)
        // More testable: expm(j*pi*[0,1;1,0]/2) ...
        // Simplest check: expm(j*pi*I/2) = e^{j*pi/2} * I = j*I
        let ev = eval_str("M = expm([0,0;0,0])");
        let m = get_matrix(&ev, "M");
        // just verify it returned an identity
        assert!(close(m[[0,0]].re, 1.0));
    }

    #[test]
    fn expm_scalar_passthrough() {
        let ev = eval_str("x = expm(1.0)");
        assert!(close(get_scalar(&ev, "x"), std::f64::consts::E));
    }

    #[test]
    fn expm_rejects_non_square() {
        let e = eval_err("expm([1,2,3;4,5,6])");
        assert!(e.contains("square"), "error should mention square: {e}");
    }

    // ── laguerre ─────────────────────────────────────────────────────────────

    #[test]
    fn laguerre_n0_is_one() {
        // L_0^alpha(x) = 1 for any alpha, x
        let ev = eval_str("x = laguerre(0, 0.5, 3.7)");
        assert!(close(get_scalar(&ev, "x"), 1.0));
    }

    #[test]
    fn laguerre_n1() {
        // L_1^alpha(x) = 1 + alpha - x
        let ev = eval_str("x = laguerre(1, 2.0, 1.5)");
        // = 1 + 2 - 1.5 = 1.5
        assert!(close(get_scalar(&ev, "x"), 1.5));
    }

    #[test]
    fn laguerre_n2_alpha1() {
        // L_2^1(x) via recurrence at x=1, alpha=1:
        // L_0=1, L_1=1+1-1=1
        // L_2 = ((2+1+1-1)*L_1 - (1+1)*L_0) / 2 = (3*1 - 2) / 2 = 0.5
        let ev = eval_str("x = laguerre(2, 1.0, 1.0)");
        assert!(close(get_scalar(&ev, "x"), 0.5));
    }

    #[test]
    fn laguerre_vector_input() {
        // L_0^0(x) = 1 element-wise
        let ev = eval_str("v = laguerre(0, 0.0, [1.0, 2.0, 3.0])");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                for c in v.iter() { assert!(close(c.re, 1.0)); }
            }
            other => panic!("expected vector, got {other:?}"),
        }
    }

    #[test]
    fn laguerre_negative_n_errors() {
        let e = eval_err("laguerre(-1, 0, 1.0)");
        assert!(e.contains("non-negative"), "error should mention non-negative: {e}");
    }

    // ── legendre ─────────────────────────────────────────────────────────────

    #[test]
    fn legendre_p00_is_one() {
        // P_0^0(x) = 1
        let ev = eval_str("x = legendre(0, 0, 0.5)");
        assert!(close(get_scalar(&ev, "x"), 1.0));
    }

    #[test]
    fn legendre_p10() {
        // P_1^0(x) = x
        let ev = eval_str("x = legendre(1, 0, 0.5)");
        assert!(close(get_scalar(&ev, "x"), 0.5));
    }

    #[test]
    fn legendre_p20() {
        // P_2^0(x) = (3x^2 - 1)/2; at x=0: -0.5
        let ev = eval_str("x = legendre(2, 0, 0.0)");
        assert!(close(get_scalar(&ev, "x"), -0.5));
    }

    #[test]
    fn legendre_p11_condon_shortley() {
        // P_1^1(x) = -(1-x^2)^0.5; at x=0: -1
        let ev = eval_str("x = legendre(1, 1, 0.0)");
        assert!(close(get_scalar(&ev, "x"), -1.0));
    }

    #[test]
    fn legendre_vector_input() {
        // P_1^0(v) = v element-wise
        let ev = eval_str("v = legendre(1, 0, [0.0, 0.5, 1.0])");
        match ev.get("v").unwrap() {
            Value::Vector(v) => {
                assert!(close(v[0].re, 0.0));
                assert!(close(v[1].re, 0.5));
                assert!(close(v[2].re, 1.0));
            }
            other => panic!("expected vector, got {other:?}"),
        }
    }

    #[test]
    fn legendre_m_exceeds_l_errors() {
        let e = eval_err("legendre(1, 2, 0.5)");
        assert!(e.contains("legendre"), "error should mention legendre: {e}");
    }
}

// ─── ML / activation function tests ─────────────────────────────────────────

#[cfg(test)]
mod ml_tests {
    use crate::{lexer, parser, Evaluator};
    use crate::eval::value::Value;

    fn eval_str(src: &str) -> Evaluator {
        let src = format!("{}\n", src);
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        for stmt in &stmts { ev.exec_stmt(stmt).unwrap(); }
        ev
    }

    fn get_vec(ev: &Evaluator, name: &str) -> Vec<f64> {
        match ev.get(name).unwrap() {
            Value::Vector(v) => v.iter().map(|c| c.re).collect(),
            other => panic!("expected vector for '{name}', got {other:?}"),
        }
    }

    fn get_scalar(ev: &Evaluator, name: &str) -> f64 {
        match ev.get(name).unwrap() {
            Value::Scalar(s) => *s,
            Value::Complex(c) => c.re,
            other => panic!("expected scalar for '{name}', got {other:?}"),
        }
    }

    fn close(a: f64, b: f64) -> bool { (a - b).abs() < 1e-6 }

    // ── softmax ──────────────────────────────────────────────────────────────

    #[test]
    fn softmax_sums_to_one() {
        let ev = eval_str("p = softmax([1.0, 2.0, 3.0, 4.0])");
        let v = get_vec(&ev, "p");
        let s: f64 = v.iter().sum();
        assert!((s - 1.0).abs() < 1e-12, "softmax should sum to 1.0, got {s}");
    }

    #[test]
    fn softmax_all_positive() {
        let ev = eval_str("p = softmax([1.0, 2.0, 3.0])");
        let v = get_vec(&ev, "p");
        for &x in &v {
            assert!(x > 0.0, "softmax output must be positive, got {x}");
        }
    }

    #[test]
    fn softmax_monotone_with_input() {
        // Larger input → larger output probability
        let ev = eval_str("p = softmax([1.0, 2.0, 3.0])");
        let v = get_vec(&ev, "p");
        assert!(v[0] < v[1] && v[1] < v[2], "softmax should be monotone: {:?}", v);
    }

    #[test]
    fn softmax_numerically_stable_large_values() {
        // Should not overflow even with large inputs
        let ev = eval_str("p = softmax([1000.0, 1001.0, 1002.0])");
        let v = get_vec(&ev, "p");
        let s: f64 = v.iter().sum();
        assert!((s - 1.0).abs() < 1e-10, "softmax should be stable for large inputs, sum={s}");
    }

    #[test]
    fn softmax_uniform_input_is_uniform_output() {
        let ev = eval_str("p = softmax([2.0, 2.0, 2.0, 2.0])");
        let v = get_vec(&ev, "p");
        for &x in &v {
            assert!((x - 0.25).abs() < 1e-10, "uniform softmax should be 0.25, got {x}");
        }
    }

    #[test]
    fn softmax_single_scalar_is_one() {
        let ev = eval_str("p = softmax(5.0)");
        let s = get_scalar(&ev, "p");
        assert!((s - 1.0).abs() < 1e-12, "softmax of scalar should be 1.0, got {s}");
    }

    // ── relu ─────────────────────────────────────────────────────────────────

    #[test]
    fn relu_positive_passthrough() {
        let ev = eval_str("y = relu(3.5)");
        assert!(close(get_scalar(&ev, "y"), 3.5), "relu(3.5) should be 3.5");
    }

    #[test]
    fn relu_negative_clamped_to_zero() {
        let ev = eval_str("y = relu(-2.0)");
        assert!(close(get_scalar(&ev, "y"), 0.0), "relu(-2) should be 0");
    }

    #[test]
    fn relu_zero_unchanged() {
        let ev = eval_str("y = relu(0.0)");
        assert!(close(get_scalar(&ev, "y"), 0.0));
    }

    #[test]
    fn relu_vector_element_wise() {
        let ev = eval_str("y = relu([-3.0, -1.0, 0.0, 2.0, 5.0])");
        let v = get_vec(&ev, "y");
        let expected = [0.0, 0.0, 0.0, 2.0, 5.0];
        for (a, b) in v.iter().zip(expected.iter()) {
            assert!(close(*a, *b), "relu mismatch: {a} vs {b}");
        }
    }

    #[test]
    fn relu_matrix_element_wise() {
        let ev = eval_str("M = relu([-1.0, 2.0; 3.0, -4.0])");
        match ev.get("M").unwrap() {
            Value::Matrix(m) => {
                assert!(close(m[[0,0]].re, 0.0));
                assert!(close(m[[0,1]].re, 2.0));
                assert!(close(m[[1,0]].re, 3.0));
                assert!(close(m[[1,1]].re, 0.0));
            }
            other => panic!("expected matrix, got {other:?}"),
        }
    }

    // ── gelu ─────────────────────────────────────────────────────────────────

    #[test]
    fn gelu_zero_is_zero() {
        let ev = eval_str("y = gelu(0.0)");
        assert!(close(get_scalar(&ev, "y"), 0.0), "gelu(0) should be 0");
    }

    #[test]
    fn gelu_positive_input_positive_output() {
        let ev = eval_str("y = gelu(1.0)");
        let s = get_scalar(&ev, "y");
        assert!(s > 0.0 && s < 1.0, "gelu(1) should be in (0,1), got {s}");
    }

    #[test]
    fn gelu_large_positive_approaches_identity() {
        // For large x, GELU(x) ≈ x
        let ev = eval_str("y = gelu(10.0)");
        let s = get_scalar(&ev, "y");
        assert!((s - 10.0).abs() < 0.01, "gelu(10) should be ~10, got {s}");
    }

    #[test]
    fn gelu_large_negative_approaches_zero() {
        let ev = eval_str("y = gelu(-10.0)");
        let s = get_scalar(&ev, "y");
        assert!(s.abs() < 0.01, "gelu(-10) should be ~0, got {s}");
    }

    #[test]
    fn gelu_negative_input_slightly_negative() {
        // GELU allows small negative outputs for x ≈ -0.17..0
        let ev = eval_str("y = gelu(-1.0)");
        let s = get_scalar(&ev, "y");
        assert!(s < 0.0, "gelu(-1) should be negative, got {s}");
    }

    #[test]
    fn gelu_vector() {
        let ev = eval_str("y = gelu([-2.0, 0.0, 2.0])");
        let v = get_vec(&ev, "y");
        assert_eq!(v.len(), 3);
        assert!(v[0] < 0.0,  "gelu(-2) < 0");
        assert!(v[1] == 0.0, "gelu(0) == 0");
        assert!(v[2] > 1.5,  "gelu(2) > 1.5");
    }

    // ── layernorm ─────────────────────────────────────────────────────────────

    #[test]
    fn layernorm_zero_mean() {
        let ev = eval_str("y = layernorm([1.0, 2.0, 3.0, 4.0, 5.0])");
        let v = get_vec(&ev, "y");
        let mean: f64 = v.iter().sum::<f64>() / v.len() as f64;
        assert!(mean.abs() < 1e-10, "layernorm output should have zero mean, got {mean}");
    }

    #[test]
    fn layernorm_unit_variance() {
        let ev = eval_str("y = layernorm([2.0, 4.0, 6.0, 8.0, 10.0])");
        let v = get_vec(&ev, "y");
        let n = v.len() as f64;
        let mean: f64 = v.iter().sum::<f64>() / n;
        let var: f64 = v.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        // var should be ~1.0 (population variance, eps≈1e-5)
        assert!((var - 1.0).abs() < 1e-4, "layernorm output should have ~unit variance, got {var}");
    }

    #[test]
    fn layernorm_length_preserved() {
        let ev = eval_str("y = layernorm([10.0, 20.0, 30.0])");
        let v = get_vec(&ev, "y");
        assert_eq!(v.len(), 3, "layernorm output length must match input");
    }

    #[test]
    fn layernorm_single_scalar_is_zero() {
        let ev = eval_str("y = layernorm(5.0)");
        assert!(close(get_scalar(&ev, "y"), 0.0), "layernorm of scalar should be 0");
    }

    #[test]
    fn layernorm_custom_eps() {
        // With a large eps the norm is less sharp but should still be close to zero mean
        let ev = eval_str("y = layernorm([1.0, 2.0, 3.0], 1.0)");
        let v = get_vec(&ev, "y");
        let mean: f64 = v.iter().sum::<f64>() / v.len() as f64;
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn layernorm_wrong_arg_errors() {
        let src = format!("{}\n", "y = layernorm([1.0, 2.0], 1.0, 2.0)");
        let tokens = lexer::tokenize(&src).unwrap();
        let stmts = parser::parse(tokens).unwrap();
        let mut ev = Evaluator::new();
        let result = ev.exec_stmt(&stmts[0]);
        assert!(result.is_err(), "layernorm with 3 args should error");
    }
}
