use syntax::{
    CalculatorError, Expr,
    Operator::{Addition, Division, Multiplication, Subtraction},
};

fn evaluator(binary: Expr) -> Result<f64, CalculatorError> {
    match binary {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left_value = evaluator(*left)?;
            let right_value = evaluator(*right)?;
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
            }
        }
        Expr::Number(value) => Ok(value),
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use syntax::{
        CalculatorError::DivisionByZero,
        Expr::{self, Number},
        Operator,
    };

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
}
