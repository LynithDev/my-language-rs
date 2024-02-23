use std::error::Error;

use crate::{create_error, create_error_list, error, lexer::token::{Token, TokenType, Tokens}, parser::ast::Literal, utils::unwrap_result};

use self::ast::{op_token_to_arithmetic, op_token_to_logical, EmptyStatement, Expression, ExpressionStatement, Node};

pub mod ast;

create_error!(TokenMismatch, {
    expected: TokenType,
    found: TokenType,
});

create_error_list!(ParserErrors, {
    TokenMismatch,
});

type ParserResult<T> = Result<T, Box<dyn Error>>;

pub struct Parser<'a> {
    pub tokens: &'a Tokens,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn from(tokens: &'a Tokens) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Node, ParserErrors> {
        Ok(Node::Program(self.parse_statements()?))
    }

    fn parse_statements(&mut self) -> ParserResult<Vec<Node>> {
        let mut statements: Vec<Node> = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> ParserResult<Node> {
        // if let Some(token) = self.get() {
        //     match token.token_type.to_owned() {
        //         TokenType::Symbol(symbol) => {
                    
        //         },
        //         _ => {}
        //     }
        // }

        Ok(self.statement()?)
    }

    fn statement(&mut self) -> ParserResult<Node> {
        if self.matches(TokenType::EndOfLine) {
            return Ok(Node::EmptyStatement(EmptyStatement()));
        }

        if self.matches(TokenType::Return) {
            return self.return_statement();
        }

        Ok(Node::ExpressionStatement(self.expression_statement()?))
    }

    fn return_statement(&mut self) -> ParserResult<Node> {
        let return_value = if !self.matches(TokenType::EndOfLine) {
            Some(self.expression()?)
        } else {
            None
        };

        if return_value.is_some() {
            // self.consume(TokenType::EndOfLine)?;
        }

        Ok(Node::ReturnStatement(ast::ReturnStatement(
            return_value
        )))
    }

    fn expression_statement(&mut self) -> ParserResult<ExpressionStatement> {
        let expression = self.expression()?;
        self.consume(TokenType::EndOfLine)?;
        Ok(ExpressionStatement(expression))
    }

    fn expression(&mut self) -> ParserResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expression> {
        let expression = self.or()?;

        if self.matches(TokenType::Equal) {
            let value = self.assignment()?;

            if let Expression::Variable(variable) = &expression {
                return Ok(Expression::Assignment(ast::Assignment(
                    variable.to_owned(),
                    Box::new(value),
                )));
            }

            error!("Invalid assignment target");
        }

        Ok(expression)
    }

    fn or(&mut self) -> ParserResult<Expression> {
        let mut expression = self.and()?;

        while self.matches(TokenType::Or) {
            let right = self.and()?;
            expression = Expression::LogicalExpression(ast::LogicalExpression(
                Box::new(expression), 
                ast::LogicalOperator::Or, 
                Box::new(right)
            ));
        }

        Ok(expression)
    }

    fn and(&mut self) -> ParserResult<Expression> {
        let mut expression = self.equality()?;

        while self.matches(TokenType::And) {
            let right = self.equality()?;
            expression = Expression::LogicalExpression(ast::LogicalExpression(
                Box::new(expression), 
                ast::LogicalOperator::And, 
                Box::new(right)
            ));
        }

        Ok(expression)
    }

    fn equality(&mut self) -> ParserResult<Expression> {
        let mut expression = self.comparison()?;

        while self.match_one_of(vec![TokenType::Equal, TokenType::NotEqual]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.comparison()?;
        
            match op_token_to_arithmetic(&operator) {
                None => error!(TokenMismatch {
                    err: format!("Expected token of type {:?}, found {:?}", TokenType::Equal, operator.token_type),
                    expected: TokenType::Equal,
                    found: operator.token_type,
                }),
                Some(op) => {
                    expression = Expression::BinaryExpression(ast::BinaryExpression(
                        Box::new(expression),
                        ast::Operator::Arithmetic(op),
                        Box::new(right),
                    ))
                }
            }
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> ParserResult<Expression> {
        let mut expression = self.addition()?;

        while self.match_one_of(vec![
            TokenType::LesserThan,
            TokenType::GreaterThan,
            TokenType::LesserThanEqual,
            TokenType::GreaterThanEqual,
        ]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.addition()?;

            let comparison_operator = unwrap_result(op_token_to_logical(&operator))?;

            expression = Expression::LogicalExpression(ast::LogicalExpression(
                Box::new(expression),
                comparison_operator,
                Box::new(right),
            ));
        }

        Ok(expression)
    }

    fn addition(&mut self) -> ParserResult<Expression> {
        let mut expression = self.multiplication()?;

        while self.match_one_of(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.multiplication()?;

            let arithmetic_operator = unwrap_result(op_token_to_arithmetic(&operator))?;

            expression = Expression::BinaryExpression(ast::BinaryExpression(
                Box::new(expression),
                ast::Operator::Arithmetic(arithmetic_operator),
                Box::new(right),
            ));
        }

        Ok(expression)
    }

    fn multiplication(&mut self) -> ParserResult<Expression> {
        let mut expression = self.unary()?;

        while self.match_one_of(vec![TokenType::Multiply, TokenType::Divide, TokenType::Modulo]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.unary()?;

            let arithmetic_operator = unwrap_result(op_token_to_arithmetic(&operator))?;

            expression = Expression::BinaryExpression(ast::BinaryExpression(
                Box::new(expression),
                ast::Operator::Arithmetic(arithmetic_operator),
                Box::new(right),
            ));
        }

        Ok(expression)
    }

    fn unary(&mut self) -> ParserResult<Expression> {
        if self.match_one_of(vec![TokenType::Minus, TokenType::Not]) {
            let operator = unwrap_result(self.previous())?.to_owned();
            let right = self.unary()?;

            let unary_operator = match operator.token_type {
                TokenType::Minus => ast::Operator::Arithmetic(ast::ArithmeticOperator::Minus),
                TokenType::Not => ast::Operator::Logical(ast::LogicalOperator::Not),
                _ => error!("bbbb")
            };

            return match unary_operator.to_owned() {
                ast::Operator::Arithmetic(_) => Ok(Expression::UnaryExpression(
                    ast::UnaryExpression(
                        unary_operator,
                        Box::new(right),
                    )
                )),
                ast::Operator::Logical(_) => Ok(Expression::UnaryExpression(
                    ast::UnaryExpression(
                        unary_operator,
                        Box::new(right),
                    )
                )),
                _ => error!("fff")
            };
        }

        self.call()
    }

    fn call(&mut self) -> ParserResult<Expression> {
        let mut expression = self.primary()?;

        loop {
            if self.matches(TokenType::LeftParen) {
                expression = self.finish_call(expression)?;
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn finish_call(&mut self, callee: Expression) -> ParserResult<Expression> {
        let mut arguments: Vec<Expression> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }

        match self.consume(TokenType::RightParen) {
            Ok(token) => token,
            Err(_) => error!(TokenMismatch {
                err: "Expected ) after arguments".to_owned(),
                expected: TokenType::RightParen,
                found: unwrap_result(self.peek())?.token_type.to_owned(),
            }),
        };

        Ok(Expression::CallExpression(
            ast::CallExpression(
                Box::new(callee),
                arguments,
            )
        ))
    }

    fn primary(&mut self) -> ParserResult<Expression> {
        let token = unwrap_result(self.peek())?.token_type.to_owned();
        let result = match token {
            TokenType::Integer(value) => {
                Ok(Expression::Literal(Literal::Integer(ast::IntegerLiteral(value))))
            },
            TokenType::Float(value) => {
                Ok(Expression::Literal(Literal::Float(ast::FloatLiteral(value))))
            },
            TokenType::Boolean(value) => {
                Ok(Expression::Literal(Literal::Boolean(ast::BooleanLiteral(value))))
            },
            _ => error!(format!("Expected expression, received '{:?}'", token)),
        };

        match result {
            Ok(expression) => {
                self.advance();
                Ok(expression)
            },
            Err(err) => Err(err),
        }
    }

    fn consume(&mut self, token: TokenType) -> ParserResult<Token> {
        if self.check(token.to_owned()) {
            return Ok(unwrap_result(self.advance())?.to_owned())
        }

        let found = unwrap_result(self.peek())?.to_owned();
        error!(TokenMismatch {
            err: format!("Expected token of type {:?}, found {:?}", token, found.token_type),
            expected: token,
            found: found.token_type,
        })
    }

    fn match_one_of(&mut self, tokens: Vec<TokenType>) -> bool {
        for token in tokens {
            if self.matches(token) {
                return true;
            }
        }

        false
    }

    fn matches(&mut self, token: TokenType) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }

        false
    }

    fn check(&self, token: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        match self.peek() {
            Some(peek) => peek.token_type == token,
            None => false,
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        match unwrap_result(self.peek()) {
            Ok(result) => {
                result.token_type == TokenType::EndOfFile
            },
            Err(_) => true,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }
    

}