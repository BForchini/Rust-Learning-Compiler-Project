use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
};
use syntax::{
    CalculatorError, Expr, FunctionDefinition,
    Operator::{Addition, Division, Equality, GreaterThan, LessThan, Multiplication, Subtraction},
    SourceProgram,
};

#[derive(Debug, Default)]
pub struct Evaluator {
    variables: HashMap<String, f64>,
    functions: HashMap<String, FunctionDefinition>,
}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<f64, CalculatorError> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        self.evaluate_with_io(&expr, &mut stdin.lock(), &mut stdout.lock())
    }

    pub fn evaluate_with_io(
        &mut self,
        expr: &Expr,
        input: &mut dyn BufRead,
        output: &mut dyn Write,
    ) -> Result<f64, CalculatorError> {
        evaluate_expr(expr, &self.functions, &mut self.variables, input, output)
    }

    pub fn evaluate_program(&mut self, program: &SourceProgram) -> Result<f64, CalculatorError> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        self.evaluate_program_with_io(program, &mut stdin.lock(), &mut stdout.lock())
    }

    pub fn evaluate_program_with_io(
        &mut self,
        program: &SourceProgram,
        input: &mut dyn BufRead,
        output: &mut dyn Write,
    ) -> Result<f64, CalculatorError> {
        let mut new_names = HashMap::new();
        for function in &program.functions {
            if matches!(function.name.as_str(), "main" | "input" | "print") {
                return Err(CalculatorError::ReservedFunctionName(function.name.clone()));
            }
            if self.functions.contains_key(&function.name)
                || new_names.insert(function.name.as_str(), ()).is_some()
            {
                return Err(CalculatorError::DuplicateFunction(function.name.clone()));
            }
        }
        for function in &program.functions {
            self.functions
                .insert(function.name.clone(), function.clone());
        }
        evaluate_expr(
            &program.expression,
            &self.functions,
            &mut self.variables,
            input,
            output,
        )
    }
}

pub fn evaluator(binary: Expr) -> Result<f64, CalculatorError> {
    Evaluator::new().evaluate(binary)
}

fn evaluate_expr(
    binary: &Expr,
    functions: &HashMap<String, FunctionDefinition>,
    variables: &mut HashMap<String, f64>,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> Result<f64, CalculatorError> {
    match binary {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left_value = evaluate_expr(left, functions, variables, input, output)?;
            let right_value = evaluate_expr(right, functions, variables, input, output)?;
            match operator {
                Addition => Ok(left_value + right_value),
                Subtraction => Ok(left_value - right_value),
                Division => {
                    if right_value != 0.0 {
                        Ok(left_value / right_value)
                    } else {
                        Err(CalculatorError::DivisionByZero)
                    }
                }
                Multiplication => Ok(left_value * right_value),
                GreaterThan => Ok(if left_value > right_value { 1.0 } else { 0.0 }),
                LessThan => Ok(if left_value < right_value { 1.0 } else { 0.0 }),
                Equality => Ok(if left_value == right_value { 1.0 } else { 0.0 }),
            }
        }
        Expr::Number(value) => Ok(*value),
        Expr::String(_) => Err(CalculatorError::InputInvalid),
        Expr::Variable(name) => variables
            .get(name)
            .copied()
            .ok_or_else(|| CalculatorError::UndefinedVariable(name.clone())),
        Expr::Assignment { name, value } => {
            let value = evaluate_expr(value, functions, variables, input, output)?;
            variables.insert(name.clone(), value);
            Ok(value)
        }
        Expr::FunctionCall { name, arguments } => {
            if name == "input" {
                if !arguments.is_empty() {
                    return Err(CalculatorError::FunctionArity {
                        name: name.clone(),
                        expected: 0,
                        actual: arguments.len(),
                    });
                }
                let mut line = String::new();
                input
                    .read_line(&mut line)
                    .map_err(|error| CalculatorError::InputOutput(error.to_string()))?;
                if line.is_empty() {
                    return Err(CalculatorError::InputOutput(
                        "no more input available".into(),
                    ));
                }
                return line
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| CalculatorError::InvalidNumberInput(line.trim().to_string()));
            }
            if name == "print" {
                if arguments.len() != 1 {
                    return Err(CalculatorError::FunctionArity {
                        name: name.clone(),
                        expected: 1,
                        actual: arguments.len(),
                    });
                }
                if let Expr::String(message) = &arguments[0] {
                    writeln!(output, "{message}")
                        .map_err(|error| CalculatorError::InputOutput(error.to_string()))?;
                    output
                        .flush()
                        .map_err(|error| CalculatorError::InputOutput(error.to_string()))?;
                    return Ok(0.0);
                }
                let value = evaluate_expr(&arguments[0], functions, variables, input, output)?;
                writeln!(output, "{value}")
                    .map_err(|error| CalculatorError::InputOutput(error.to_string()))?;
                output
                    .flush()
                    .map_err(|error| CalculatorError::InputOutput(error.to_string()))?;
                return Ok(value);
            }
            let function = functions
                .get(name)
                .cloned()
                .ok_or_else(|| CalculatorError::UndefinedFunction(name.clone()))?;
            if arguments.len() != function.parameters.len() {
                return Err(CalculatorError::FunctionArity {
                    name: name.clone(),
                    expected: function.parameters.len(),
                    actual: arguments.len(),
                });
            }

            let mut local_variables = HashMap::new();
            for (parameter, argument) in function.parameters.iter().zip(arguments) {
                let value = evaluate_expr(argument, functions, variables, input, output)?;
                local_variables.insert(parameter.clone(), value);
            }
            evaluate_expr(
                &function.body,
                functions,
                &mut local_variables,
                input,
                output,
            )
        }
        Expr::Block(expressions) => {
            let mut result = 0.0;
            for expression in expressions {
                result = evaluate_expr(expression, functions, variables, input, output)?;
            }
            Ok(result)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = evaluate_expr(condition, functions, variables, input, output)?;
            if condition != 0.0 {
                evaluate_expr(then_branch, functions, variables, input, output)
            } else {
                evaluate_expr(else_branch, functions, variables, input, output)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use std::io::Cursor;
    use syntax::{
        CalculatorError::{DivisionByZero, InputOutput, InvalidNumberInput},
        Expr::{self, Number},
        Operator,
    };

    #[test]
    fn evaluates_assignment() {
        let assignment = Expr::Assignment {
            name: "x".into(),
            value: Box::new(Expr::Binary {
                left: Box::new(Number(4.0)),
                operator: Addition,
                right: Box::new(Number(3.0)),
            }),
        };

        assert_eq!(evaluator(assignment), Ok(7.0));
    }

    #[test]
    fn evaluates_reference_in_a_later_expression() {
        let mut evaluator = Evaluator::new();

        assert_eq!(
            evaluator.evaluate(Expr::Assignment {
                name: "x".into(),
                value: Box::new(Expr::Binary {
                    left: Box::new(Number(4.0)),
                    operator: Addition,
                    right: Box::new(Number(3.0)),
                }),
            }),
            Ok(7.0)
        );
        assert_eq!(
            evaluator.evaluate(Expr::Binary {
                left: Box::new(Expr::Variable("x".into())),
                operator: Addition,
                right: Box::new(Number(1.0)),
            }),
            Ok(8.0)
        );
    }

    #[test]
    fn evaluates_nested_function_calls() {
        let program = SourceProgram {
            functions: vec![
                FunctionDefinition {
                    name: "add".into(),
                    parameters: vec!["a".into(), "b".into()],
                    body: Expr::Binary {
                        left: Box::new(Expr::Variable("a".into())),
                        operator: Addition,
                        right: Box::new(Expr::Variable("b".into())),
                    },
                },
                FunctionDefinition {
                    name: "twice".into(),
                    parameters: vec!["x".into()],
                    body: Expr::FunctionCall {
                        name: "add".into(),
                        arguments: vec![Expr::Variable("x".into()), Expr::Variable("x".into())],
                    },
                },
            ],
            expression: Expr::FunctionCall {
                name: "twice".into(),
                arguments: vec![Number(4.0)],
            },
        };

        assert_eq!(Evaluator::new().evaluate_program(&program), Ok(8.0));
    }

    #[test]
    fn simple_star() {
        let binary = Expr::Binary {
            left: Box::new(Expr::Number(3.0)),
            operator: Operator::Multiplication,
            right: Box::new(Expr::Number(5.0)),
        };

        assert_eq!(evaluator(binary), Ok(15.0));
    }
    #[test]
    fn simple_addition() {
        let binary = Expr::Binary {
            left: Box::new(Number(3.0)),
            operator: Subtraction,
            right: Box::new(Number(5.0)),
        };

        assert_eq!(evaluator(binary), Ok(-2.0))
    }
    #[test]
    fn simple_division() {
        let binary = Expr::Binary {
            left: Box::new(Number(3.0)),
            operator: Division,
            right: Box::new(Number(5.0)),
        };

        assert_eq!(evaluator(binary), Ok(0.6));
    }
    #[test]
    fn star_then_add() {
        let binary = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Multiplication,
                right: Box::new(Number(5.0)),
            }),
            operator: Addition,
            right: Box::new(Number(2.0)),
        };

        assert_eq!(evaluator(binary), Ok(17.0));
    }
    #[test]
    fn add_then_star() {
        let binary = Expr::Binary {
            left: Box::new(Number(3.0)),
            operator: Addition,
            right: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Multiplication,
                right: Box::new(Number(5.0)),
            }),
        };

        assert_eq!(evaluator(binary), Ok(18.0));
    }
    #[test]
    fn add_then_star_then_sub() {
        let binary = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Number(3.0)),
                operator: Addition,
                right: Box::new(Expr::Binary {
                    left: Box::new(Number(5.0)),
                    operator: Multiplication,
                    right: Box::new(Number(2.0)),
                }),
            }),
            operator: Subtraction,
            right: Box::new(Number(4.0)),
        };

        assert_eq!(evaluator(binary), Ok(9.0));
    }

    #[test]
    fn division_by_zero() {
        let binary = Expr::Binary {
            left: Box::new(Number(2.0)),
            operator: Division,
            right: Box::new(Number(0.0)),
        };

        assert_eq!(evaluator(binary), Err(DivisionByZero));
    }

    #[test]
    fn evaluates_input_and_print_calls() {
        let expression = Expr::Block(vec![
            Expr::FunctionCall {
                name: "print".into(),
                arguments: vec![Expr::String("Enter a value:".into())],
            },
            Expr::Assignment {
                name: "value".into(),
                value: Box::new(Expr::FunctionCall {
                    name: "input".into(),
                    arguments: Vec::new(),
                }),
            },
            Expr::FunctionCall {
                name: "print".into(),
                arguments: vec![Expr::Variable("value".into())],
            },
            Expr::Variable("value".into()),
        ]);
        let mut evaluator = Evaluator::new();
        let mut input = Cursor::new(b"6.5\n");
        let mut output = Vec::new();

        assert_eq!(
            evaluator.evaluate_with_io(&expression, &mut input, &mut output),
            Ok(6.5)
        );
        assert_eq!(String::from_utf8(output).unwrap(), "Enter a value:\n6.5\n");
    }

    #[test]
    fn evaluates_only_the_selected_if_branch() {
        let expression = Expr::If {
            condition: Box::new(Number(0.0)),
            then_branch: Box::new(Expr::FunctionCall {
                name: "print".into(),
                arguments: vec![Expr::String("then".into())],
            }),
            else_branch: Box::new(Expr::Block(vec![
                Expr::FunctionCall {
                    name: "print".into(),
                    arguments: vec![Expr::String("else".into())],
                },
                Number(7.0),
            ])),
        };
        let mut evaluator = Evaluator::new();
        let mut input = Cursor::new(b"");
        let mut output = Vec::new();

        assert_eq!(
            evaluator.evaluate_with_io(&expression, &mut input, &mut output),
            Ok(7.0)
        );
        assert_eq!(String::from_utf8(output).unwrap(), "else\n");
    }

    #[test]
    fn invalid_input_returns_number_error() {
        let expression = Expr::FunctionCall {
            name: "input".into(),
            arguments: Vec::new(),
        };
        let mut evaluator = Evaluator::new();
        let mut input = Cursor::new(b"not-a-number\n");
        let mut output = Vec::new();

        assert_eq!(
            evaluator.evaluate_with_io(&expression, &mut input, &mut output),
            Err(InvalidNumberInput("not-a-number".into()))
        );
    }

    #[test]
    fn missing_input_returns_io_error() {
        let expression = Expr::FunctionCall {
            name: "input".into(),
            arguments: Vec::new(),
        };
        let mut evaluator = Evaluator::new();
        let mut input = Cursor::new(b"");
        let mut output = Vec::new();

        assert_eq!(
            evaluator.evaluate_with_io(&expression, &mut input, &mut output),
            Err(InputOutput("no more input available".into()))
        );
    }
}
