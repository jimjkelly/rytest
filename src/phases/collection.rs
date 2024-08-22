use anyhow::Result;
use glob::glob;
use pyo3::exceptions::PySyntaxError;
use pyo3::PyErr;
use rustpython_parser::ast::Stmt::{self, ClassDef, FunctionDef};
use rustpython_parser::{ast, Parse};
use std::io::Read;
use std::{fs::File, sync::mpsc};

use crate::TestCase;

use crate::phases::collectors::ignore_test;

pub fn find_files(paths: Vec<String>, prefix: &str, tx: mpsc::Sender<String>) -> Result<()> {
    for path in &paths {
        for entry in glob(path.as_str())? {
            match entry {
                Ok(p) => {
                    if p.is_file()
                        && p.file_stem().unwrap().to_string_lossy().starts_with(prefix)
                        && p.extension().unwrap() == "py"
                    {
                        tx.send(p.to_str().unwrap().to_string())?;
                    }
                }
                Err(e) => println!("Error globbing: {}", e),
            }
        }
    }

    drop(tx);

    Ok(())
}

pub fn find_tests(
    prefix: String,
    verbose: bool,
    rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<TestCase>,
) -> Result<()> {
    while let Ok(file_name) = rx.recv() {
        let mut data = String::new();
        let mut file = File::open(file_name.clone())?;
        file.read_to_string(&mut data)?;
        let ast = ast::Suite::parse(data.as_str(), "<embedded>");

        match ast {
            Ok(ast) => {
                for stmt in ast {
                    match stmt {
                        FunctionDef(ref node)
                            if node.name.starts_with(&prefix)
                                && !ignore_test::is_pytest_fixture(stmt.clone()) =>
                        {
                            tx.send(TestCase {
                                file: file_name.clone(),
                                name: node.name.to_string(),
                                passed: false,
                                error: None,
                            })?
                        }
                        ClassDef(node) if node.bases.iter().any(find_unittest_base) => {
                            let cases = find_unittest_class_cases(
                                node.body.clone(),
                                &prefix,
                                file_name.clone(),
                                verbose,
                            );
                            if !cases.is_empty() {
                                for case in cases {
                                    let class = node.name.as_str();
                                    tx.send(TestCase {
                                        file: file_name.clone(),
                                        name: format!("{}::{}", class, case),
                                        passed: false,
                                        error: None,
                                    })?
                                }
                            }
                        }
                        _ if verbose => println!("{}: Skipping {:#?}\n\n", file_name, stmt),
                        _ => {}
                    }
                }
            }
            Err(e) => tx.send(TestCase {
                file: file_name.clone(),
                name: "".to_string(),
                passed: false,
                error: Some(PyErr::new::<PySyntaxError, _>(format!(
                    " Error parsing {}",
                    e
                ))),
            })?,
        }
    }

    Ok(())
}

fn find_unittest_base(expr: &ast::Expr) -> bool {
    if expr.is_attribute_expr() {
        let attr_expr = expr.as_attribute_expr().unwrap();
        let module = attr_expr.value.as_name_expr().unwrap().id.as_str();
        module == "unittest" && attr_expr.attr.as_str() == "TestCase"
    } else {
        false
    }
}

fn find_unittest_class_cases(
    stmts: Vec<Stmt>,
    prefix: &str,
    file_name: String,
    verbose: bool,
) -> Vec<String> {
    let mut cases = vec![];
    for stmt in stmts {
        match stmt {
            FunctionDef(node)
                if node.name.starts_with(prefix)
                    && !ignore_test::is_pytest_fixture(stmt.clone()) =>
            {
                cases.push(node.name.to_string())
            }
            _ if verbose => println!("{}: Skipping {:#?}\n\n", file_name, stmt),
            _ => {}
        }
    }
    cases
}

#[cfg(test)]
mod tests {
    use ast::{text_size::TextRange, EmptyRange, Expr, Identifier, TextSize};

    use super::*;

    #[test]
    fn test_find_unittest_base() {
        let expr = Expr::Attribute(ast::ExprAttribute {
            range: TextRange::new(TextSize::from(34), TextSize::from(51)),
            value: Box::new(Expr::Name(ast::ExprName {
                range: TextRange::new(TextSize::from(34), TextSize::from(42)),
                id: Identifier::new("unittest".to_string()),
                ctx: ast::ExprContext::Load,
            })),
            attr: Identifier::new("TestCase".to_string()),
            ctx: ast::ExprContext::Load,
        });
        assert!(find_unittest_base(&expr));
    }

    #[test]
    fn test_find_unittest_base_false() {
        let expr = Expr::Attribute(ast::ExprAttribute {
            range: TextRange::new(TextSize::from(34), TextSize::from(51)),
            value: Box::new(Expr::Name(ast::ExprName {
                range: TextRange::new(TextSize::from(34), TextSize::from(42)),
                id: Identifier::new("unittest".to_string()),
                ctx: ast::ExprContext::Load,
            })),
            attr: Identifier::new("NotTestCase".to_string()),
            ctx: ast::ExprContext::Load,
        });
        assert!(!find_unittest_base(&expr));
    }

    #[test]
    fn test_find_unittest_base_false_no_attribute_expr() {
        let expr = Expr::Name(ast::ExprName {
            range: TextRange::new(TextSize::from(34), TextSize::from(42)),
            id: Identifier::new("unittest".to_string()),
            ctx: ast::ExprContext::Load,
        });
        assert!(!find_unittest_base(&expr));
    }

    #[test]
    fn test_find_unittest_class_cases() {
        let stmts = vec![
            FunctionDef(ast::StmtFunctionDef {
                range: TextRange::new(TextSize::from(0), TextSize::from(0)),
                name: Identifier::new("test_one".to_string()),
                args: Box::new(ast::Arguments {
                    range: EmptyRange::default(),
                    posonlyargs: vec![],
                    args: vec![],
                    vararg: None,
                    kwonlyargs: vec![],
                    kwarg: None,
                }),
                body: vec![],
                decorator_list: vec![],
                type_params: vec![],
                returns: None,
                type_comment: None,
            }),
            FunctionDef(ast::StmtFunctionDef {
                range: TextRange::new(TextSize::from(0), TextSize::from(0)),
                name: Identifier::new("not_a_test".to_string()),
                args: Box::new(ast::Arguments {
                    range: EmptyRange::default(),
                    posonlyargs: vec![],
                    args: vec![],
                    vararg: None,
                    kwonlyargs: vec![],
                    kwarg: None,
                }),
                body: vec![],
                decorator_list: vec![],
                type_params: vec![],
                returns: None,
                type_comment: None,
            }),
            FunctionDef(ast::StmtFunctionDef {
                range: TextRange::new(TextSize::from(0), TextSize::from(0)),
                name: Identifier::new("test_two".to_string()),
                args: Box::new(ast::Arguments {
                    range: EmptyRange::default(),
                    posonlyargs: vec![],
                    args: vec![],
                    vararg: None,
                    kwonlyargs: vec![],
                    kwarg: None,
                }),
                body: vec![],
                decorator_list: vec![],
                type_params: vec![],
                returns: None,
                type_comment: None,
            }),
        ];
        let prefix = "test_";
        let file_name = "test.py".to_string();
        let verbose = false;
        assert_eq!(
            find_unittest_class_cases(stmts, prefix, file_name, verbose),
            vec!["test_one".to_string(), "test_two".to_string()]
        );
    }

    #[test]
    fn test_find_unittest_class_cases_empty() {
        let stmts = vec![
            FunctionDef(ast::StmtFunctionDef {
                range: TextRange::new(TextSize::from(0), TextSize::from(0)),
                name: Identifier::new("tests_one".to_string()),
                args: Box::new(ast::Arguments {
                    range: EmptyRange::default(),
                    posonlyargs: vec![],
                    args: vec![],
                    vararg: None,
                    kwonlyargs: vec![],
                    kwarg: None,
                }),
                body: vec![],
                decorator_list: vec![],
                type_params: vec![],
                returns: None,
                type_comment: None,
            }),
            FunctionDef(ast::StmtFunctionDef {
                range: TextRange::new(TextSize::from(0), TextSize::from(0)),
                name: Identifier::new("not_a_test".to_string()),
                args: Box::new(ast::Arguments {
                    range: EmptyRange::default(),
                    posonlyargs: vec![],
                    args: vec![],
                    vararg: None,
                    kwonlyargs: vec![],
                    kwarg: None,
                }),
                body: vec![],
                decorator_list: vec![],
                type_params: vec![],
                returns: None,
                type_comment: None,
            }),
            FunctionDef(ast::StmtFunctionDef {
                range: TextRange::new(TextSize::from(0), TextSize::from(0)),
                name: Identifier::new("tests_two".to_string()),
                args: Box::new(ast::Arguments {
                    range: EmptyRange::default(),
                    posonlyargs: vec![],
                    args: vec![],
                    vararg: None,
                    kwonlyargs: vec![],
                    kwarg: None,
                }),
                body: vec![],
                decorator_list: vec![],
                type_params: vec![],
                returns: None,
                type_comment: None,
            }),
        ];
        let prefix = "test_";
        let file_name = "test.py".to_string();
        let verbose = false;
        assert_eq!(
            find_unittest_class_cases(stmts, prefix, file_name, verbose),
            Vec::<String>::new()
        );
    }
}
