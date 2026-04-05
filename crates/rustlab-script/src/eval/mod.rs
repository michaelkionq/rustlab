pub mod builtins;
pub mod value;

use std::collections::HashMap;
use ndarray::Array1;
use num_complex::Complex;
use rustlab_core::C64;
use crate::ast::{Expr, Stmt};
use crate::error::ScriptError;
pub use value::Value;
pub use builtins::BuiltinRegistry;

#[derive(Clone)]
struct UserFn {
    params:     Vec<String>,
    return_var: Option<String>,
    body:       Vec<Stmt>,
}

pub struct Evaluator {
    env:           HashMap<String, Value>,
    builtins:      BuiltinRegistry,
    user_fns:      HashMap<String, UserFn>,
    /// True while executing a user-defined function body — suppresses auto-print of assignments.
    in_function:   bool,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        // Predefined constants: i and j both equal Complex(0, 1)
        env.insert("j".to_string(), Value::Complex(num_complex::Complex::new(0.0, 1.0)));
        env.insert("i".to_string(), Value::Complex(num_complex::Complex::new(0.0, 1.0)));
        // Also pi and e for convenience
        env.insert("pi".to_string(), Value::Scalar(std::f64::consts::PI));
        env.insert("e".to_string(),  Value::Scalar(std::f64::consts::E));

        Self {
            env,
            builtins:    BuiltinRegistry::with_defaults(),
            user_fns:    HashMap::new(),
            in_function: false,
        }
    }

    /// Look up a variable in the environment (used by tests).
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.env.get(name)
    }

    /// Remove all user-defined variables, keeping built-in constants (j, pi, e).
    pub fn clear_vars(&mut self) {
        const BUILTIN_CONSTS: &[&str] = &["i", "j", "pi", "e"];
        self.env.retain(|k, _| BUILTIN_CONSTS.contains(&k.as_str()));
    }

    /// Return all user-defined variables, sorted by name.
    /// Excludes built-in constants (j, pi, e).
    pub fn vars(&self) -> Vec<(&str, &Value)> {
        const BUILTIN_CONSTS: &[&str] = &["i", "j", "pi", "e"];
        let mut entries: Vec<(&str, &Value)> = self.env.iter()
            .filter(|(k, _)| !BUILTIN_CONSTS.contains(&k.as_str()))
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        entries.sort_by_key(|(k, _)| *k);
        entries
    }

    pub fn run(&mut self, stmts: &[Stmt]) -> Result<(), ScriptError> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn exec_stmt(&mut self, stmt: &Stmt) -> Result<(), ScriptError> {
        match stmt {
            Stmt::Assign { name, expr, suppress } => {
                let val = self.eval_expr(expr)?;
                if !suppress && !self.in_function {
                    println!("{} = {}", name, val);
                }
                self.env.insert(name.clone(), val);
            }
            Stmt::FunctionDef { name, params, return_var, body } => {
                self.user_fns.insert(name.clone(), UserFn {
                    params:     params.clone(),
                    return_var: return_var.clone(),
                    body:       body.clone(),
                });
            }
            Stmt::FieldAssign { object, field, expr, suppress } => {
                let val = self.eval_expr(expr)?;
                if !suppress && !self.in_function {
                    println!("{}.{} = {}", object, field, val);
                }
                match self.env.get_mut(object) {
                    Some(Value::Struct(fields)) => {
                        fields.insert(field.clone(), val);
                    }
                    Some(other) => {
                        return Err(ScriptError::Runtime(format!(
                            "'{}' is a {}, not a struct", object, other.type_name()
                        )));
                    }
                    None => {
                        // Auto-create a new struct when assigning to unknown.field
                        let mut fields = HashMap::new();
                        fields.insert(field.clone(), val);
                        self.env.insert(object.clone(), Value::Struct(fields));
                    }
                }
            }
            Stmt::Return => {
                return Err(ScriptError::EarlyReturn);
            }
            Stmt::If { cond, then_body, else_body } => {
                let cv = self.eval_expr(cond)?;
                let branch = match cv {
                    Value::Bool(b) => b,
                    Value::Scalar(n) => n != 0.0,
                    other => return Err(ScriptError::Runtime(format!(
                        "if condition must be a bool or scalar, got {}", other.type_name()
                    ))),
                };
                let body = if branch { then_body } else { else_body };
                for s in body {
                    self.exec_stmt(s)?;
                }
            }
            Stmt::MultiAssign { names, expr, suppress } => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Tuple(values) => {
                        if values.len() < names.len() {
                            return Err(ScriptError::Runtime(format!(
                                "multi-assign: expected {} values, function returned {}",
                                names.len(), values.len()
                            )));
                        }
                        for (name, v) in names.iter().zip(values.into_iter()) {
                            if name == "~" { continue; } // discard
                            if !suppress && !self.in_function {
                                println!("{} = {}", name, v);
                            }
                            self.env.insert(name.clone(), v);
                        }
                    }
                    single => {
                        if names.len() != 1 {
                            return Err(ScriptError::Runtime(format!(
                                "multi-assign: expected {} values, function returned 1",
                                names.len()
                            )));
                        }
                        if names[0] != "~" {
                            if !suppress && !self.in_function {
                                println!("{} = {}", names[0], single);
                            }
                            self.env.insert(names[0].clone(), single);
                        }
                    }
                }
            }
            Stmt::For { var, iter, body } => {
                let iter_val = self.eval_expr(iter)?;
                let elements = match iter_val {
                    Value::Vector(v) => v.to_vec(),
                    Value::Scalar(n) => vec![Complex::new(n, 0.0)],
                    other => return Err(ScriptError::Runtime(format!(
                        "for: cannot iterate over {}", other.type_name()
                    ))),
                };
                for elem in elements {
                    let val = if elem.im == 0.0 {
                        Value::Scalar(elem.re)
                    } else {
                        Value::Complex(elem)
                    };
                    self.env.insert(var.clone(), val);
                    for s in body {
                        self.exec_stmt(s)?;
                    }
                }
            }
            Stmt::IndexAssign { name, indices, expr, suppress } => {
                let val = self.eval_expr(expr)?;

                // Evaluate indices with `end` bound to current container length (if any)
                let container_len = match self.env.get(name.as_str()) {
                    Some(Value::Vector(v)) => v.len(),
                    Some(Value::Matrix(m)) if indices.len() == 1 => m.nrows() * m.ncols(),
                    _ => 0,
                };
                self.env.insert("end".to_string(), Value::Scalar(container_len as f64));
                let idx_vals: Vec<Value> = indices.iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
                self.env.remove("end");

                if idx_vals.len() == 1 {
                    // Single-index: vector assignment (auto-create/grow)
                    let idx = idx_vals[0].to_scalar()
                        .map_err(ScriptError::Type)? as usize;
                    if idx < 1 {
                        return Err(ScriptError::Runtime(
                            "index assignment: index must be >= 1".to_string()
                        ));
                    }
                    let assign_val = match &val {
                        Value::Scalar(n)  => Complex::new(*n, 0.0),
                        Value::Complex(c) => *c,
                        other => return Err(ScriptError::Runtime(format!(
                            "index assignment: right-hand side must be scalar or complex, got {}",
                            other.type_name()
                        ))),
                    };
                    let vec = match self.env.get_mut(name.as_str()) {
                        Some(Value::Vector(v)) => {
                            if idx > v.len() {
                                let mut new_vec = vec![Complex::new(0.0, 0.0); idx];
                                for (i, c) in v.iter().enumerate() { new_vec[i] = *c; }
                                *v = Array1::from_vec(new_vec);
                            }
                            v
                        }
                        _ => {
                            // Create new vector of length idx, filled with zeros
                            let new_vec = vec![Complex::new(0.0, 0.0); idx];
                            self.env.insert(name.clone(), Value::Vector(Array1::from_vec(new_vec)));
                            match self.env.get_mut(name.as_str()) {
                                Some(Value::Vector(v)) => v,
                                _ => unreachable!(),
                            }
                        }
                    };
                    vec[idx - 1] = assign_val;
                    if !suppress && !self.in_function {
                        println!("{}({}) = {}", name, idx, Value::Complex(assign_val));
                    }
                } else if idx_vals.len() == 2 {
                    // Two-index: matrix assignment
                    let row = idx_vals[0].to_scalar().map_err(ScriptError::Type)? as usize;
                    let col = idx_vals[1].to_scalar().map_err(ScriptError::Type)? as usize;
                    if row < 1 || col < 1 {
                        return Err(ScriptError::Runtime(
                            "index assignment: indices must be >= 1".to_string()
                        ));
                    }
                    let assign_val = match &val {
                        Value::Scalar(n)  => Complex::new(*n, 0.0),
                        Value::Complex(c) => *c,
                        other => return Err(ScriptError::Runtime(format!(
                            "index assignment: right-hand side must be scalar or complex, got {}",
                            other.type_name()
                        ))),
                    };
                    match self.env.get_mut(name.as_str()) {
                        Some(Value::Matrix(m)) => {
                            if row > m.nrows() || col > m.ncols() {
                                return Err(ScriptError::Runtime(format!(
                                    "index assignment: ({},{}) out of bounds for {}×{} matrix",
                                    row, col, m.nrows(), m.ncols()
                                )));
                            }
                            m[[row - 1, col - 1]] = assign_val;
                            if !suppress && !self.in_function {
                                println!("{}({},{}) = {}", name, row, col, Value::Complex(assign_val));
                            }
                        }
                        _ => return Err(ScriptError::Runtime(format!(
                            "index assignment: '{}' is not a matrix", name
                        ))),
                    }
                } else {
                    return Err(ScriptError::Runtime(
                        "index assignment: only 1 or 2 indices are supported".to_string()
                    ));
                }
            }
            Stmt::Expr(expr, suppress) => {
                // Special case: bare load("file.npz") injects all variables into the workspace.
                if let Expr::Call { name, args } = expr {
                    if name == "load" && args.len() == 1 {
                        let path_val = self.eval_expr(&args[0])?;
                        if let Ok(path) = path_val.to_str() {
                            if path.ends_with(".npz") {
                                let vars = builtins::load_all_from_npz(&path)
                                    .map_err(ScriptError::Runtime)?;
                                if !suppress {
                                    let names: Vec<&str> = vars.iter().map(|(n, _)| n.as_str()).collect();
                                    println!("loaded: {}", names.join(", "));
                                }
                                for (var_name, val) in vars {
                                    self.env.insert(var_name, val);
                                }
                                return Ok(());
                            }
                        }
                    }
                }

                let val = self.eval_expr(expr)?;
                if !suppress && !self.in_function && !matches!(val, Value::None) {
                    println!("{}", val);
                }
            }
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, ScriptError> {
        match expr {
            Expr::Number(n) => Ok(Value::Scalar(*n)),
            Expr::Str(s)    => Ok(Value::Str(s.clone())),
            Expr::Var(name) => {
                self.env.get(name)
                    .cloned()
                    .ok_or_else(|| ScriptError::Undefined(name.clone()))
            }
            Expr::UnaryMinus(inner) => {
                let v = self.eval_expr(inner)?;
                v.negate().map_err(ScriptError::Type)
            }
            Expr::UnaryNot(inner) => {
                let v = self.eval_expr(inner)?;
                v.not().map_err(ScriptError::Type)
            }
            Expr::BinOp { op, lhs, rhs } => {
                let l = self.eval_expr(lhs)?;
                let r = self.eval_expr(rhs)?;
                Value::binop(*op, l, r).map_err(ScriptError::Type)
            }
            Expr::Call { name, args } => {
                // If the name refers to a vector/matrix in the environment, this is indexing.
                if matches!(self.env.get(name.as_str()), Some(Value::Vector(_)) | Some(Value::Matrix(_))) {
                    let container = self.env[name.as_str()].clone();

                    // For 2-argument matrix indexing, bind `end` context-sensitively per dimension.
                    let idx_vals: Vec<Value> = if args.len() == 2 {
                        if let Value::Matrix(m) = &container {
                            let nrows = m.nrows();
                            let ncols = m.ncols();
                            self.env.insert("end".to_string(), Value::Scalar(nrows as f64));
                            let row_val = self.eval_expr(&args[0])?;
                            self.env.insert("end".to_string(), Value::Scalar(ncols as f64));
                            let col_val = self.eval_expr(&args[1])?;
                            self.env.remove("end");
                            vec![row_val, col_val]
                        } else {
                            let len = match &container {
                                Value::Vector(v) => v.len(),
                                _ => unreachable!(),
                            };
                            self.env.insert("end".to_string(), Value::Scalar(len as f64));
                            let vals: Vec<Value> = args.iter()
                                .map(|a| self.eval_expr(a))
                                .collect::<Result<_, _>>()?;
                            self.env.remove("end");
                            vals
                        }
                    } else {
                        let len = match &container {
                            Value::Vector(v) => v.len(),
                            Value::Matrix(m) => m.nrows(),
                            _ => unreachable!(),
                        };
                        self.env.insert("end".to_string(), Value::Scalar(len as f64));
                        let vals: Vec<Value> = args.iter()
                            .map(|a| self.eval_expr(a))
                            .collect::<Result<_, _>>()?;
                        self.env.remove("end");
                        vals
                    };

                    container.index(idx_vals).map_err(ScriptError::Runtime)
                } else if let Some(func) = self.user_fns.get(name.as_str()).cloned() {
                    let vals: Vec<Value> = args.iter()
                        .map(|a| self.eval_expr(a))
                        .collect::<Result<_, _>>()?;
                    self.eval_user_fn(func, vals)
                } else {
                    let vals: Vec<Value> = args.iter()
                        .map(|a| self.eval_expr(a))
                        .collect::<Result<_, _>>()?;
                    self.builtins.call(name, vals)
                }
            }
            Expr::Matrix(rows) => {
                let evaled: Vec<Vec<Value>> = rows.iter()
                    .map(|row| row.iter().map(|e| self.eval_expr(e)).collect::<Result<_, _>>())
                    .collect::<Result<_, _>>()?;
                Value::from_matrix_rows(evaled).map_err(ScriptError::Type)
            }
            Expr::Range { start, step, stop } => {
                let s = self.eval_expr(start)?.to_scalar().map_err(ScriptError::Type)?;
                let e = self.eval_expr(stop)?.to_scalar().map_err(ScriptError::Type)?;
                let inc = match step {
                    Some(st) => self.eval_expr(st)?.to_scalar().map_err(ScriptError::Type)?,
                    None     => 1.0,
                };
                if inc == 0.0 {
                    return Err(ScriptError::Runtime("range step cannot be zero".to_string()));
                }
                let mut vals: Vec<C64> = Vec::new();
                let mut cur = s;
                // Use a small epsilon to avoid float boundary issues
                let eps = inc.abs() * 1e-10;
                if inc > 0.0 {
                    while cur <= e + eps { vals.push(Complex::new(cur, 0.0)); cur += inc; }
                } else {
                    while cur >= e - eps { vals.push(Complex::new(cur, 0.0)); cur += inc; }
                }
                Ok(Value::Vector(Array1::from_vec(vals)))
            }
            Expr::Transpose(inner) => {
                let v = self.eval_expr(inner)?;
                v.transpose().map_err(ScriptError::Runtime)
            }
            Expr::NonConjTranspose(inner) => {
                let v = self.eval_expr(inner)?;
                v.non_conj_transpose().map_err(ScriptError::Runtime)
            }
            Expr::All => Ok(Value::All),
            Expr::Index { expr, args } => {
                let container = self.eval_expr(expr)?;
                // Bind `end` to length of the container for use inside index expressions
                let end_val = match &container {
                    Value::Vector(v) => v.len(),
                    Value::Matrix(m) => m.nrows(),
                    _ => 0,
                };
                self.env.insert("end".to_string(), Value::Scalar(end_val as f64));
                let idx_vals: Vec<Value> = args.iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
                self.env.remove("end");
                container.index(idx_vals).map_err(ScriptError::Runtime)
            }
            Expr::Field { object, field } => {
                let obj = self.eval_expr(object)?;
                match obj {
                    Value::Struct(fields) => fields.get(field.as_str())
                        .cloned()
                        .ok_or_else(|| ScriptError::Runtime(
                            format!("struct has no field '{}'", field)
                        )),
                    Value::StateSpace { a, b, c, d } => match field.as_str() {
                        "A" => Ok(Value::Matrix(a)),
                        "B" => Ok(Value::Matrix(b)),
                        "C" => Ok(Value::Matrix(c)),
                        "D" => Ok(Value::Matrix(d)),
                        other => Err(ScriptError::Runtime(format!(
                            "ss has no field '{}'; valid fields are A, B, C, D", other
                        ))),
                    },
                    other => Err(ScriptError::Runtime(format!(
                        "cannot access field '{}' on {}", field, other.type_name()
                    ))),
                }
            }
        }
    }

    /// Call a user-defined function with scope isolation.
    fn eval_user_fn(&mut self, func: UserFn, args: Vec<Value>) -> Result<Value, ScriptError> {
        if args.len() != func.params.len() {
            return Err(ScriptError::Runtime(format!(
                "function expects {} argument(s), got {}", func.params.len(), args.len()
            )));
        }
        // Save outer env and function flag
        let saved_env = std::mem::take(&mut self.env);
        let saved_in_fn = self.in_function;
        self.in_function = true;
        // Seed with built-in constants
        for name in &["i", "j", "pi", "e"] {
            if let Some(v) = saved_env.get(*name) {
                self.env.insert((*name).to_string(), v.clone());
            }
        }
        // Bind parameters
        for (param, val) in func.params.iter().zip(args) {
            self.env.insert(param.clone(), val);
        }
        // Run body — EarlyReturn is not an error, just early exit
        let run_result = self.run(&func.body);
        if let Err(ScriptError::EarlyReturn) = run_result {
            // normal early return — continue to extract return value
        } else {
            run_result?;
        }
        // Extract return value from function scope
        let ret_val = if let Some(ref ret) = func.return_var {
            self.env.get(ret.as_str()).cloned().unwrap_or(Value::None)
        } else {
            Value::None
        };
        // Restore outer env and function flag
        self.env = saved_env;
        self.in_function = saved_in_fn;
        Ok(ret_val)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
