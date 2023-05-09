//! Собственные ошибки для модуля world.

use std::error::Error;
use std::fmt;

/// Исправимые ошибки.
#[derive(Debug, Clone)]
pub struct RecoverableError {
    description: String,
}

impl RecoverableError {
    pub(crate) fn new(message: String) -> RecoverableError {
        RecoverableError {
            description: message,
        }
    }
}

impl fmt::Display for RecoverableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for RecoverableError {}

/// Ошибки связанные с добавлением агента.
#[derive(Debug)]
pub enum AddAgentError {
    TakenCell((usize, usize)),
    OutOfBounds((usize, usize)),
    //Overpopulated,
}

impl fmt::Display for AddAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddAgentError::TakenCell(point) => write!(
                f, "В ячейке с координатами ({}, {}) уже содержит агент (растение или животное)", point.0, point.1
            ),
            AddAgentError::OutOfBounds(point) => write!(
                f, "Точка с координатами ({}, {}) выходит за границы мира", point.0, point.1
            ),
            // AddAgentError::Overpopulated =>  write!(
            //     f, "Мир перенаселен, в нем закончилось место для новых животных"
            // ),
        }
    }
}

impl Error for AddAgentError {}