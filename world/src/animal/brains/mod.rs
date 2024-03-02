//! Модуль, реализующий общие методы мозга животного.

pub(crate) mod simple;

use crate::animal::{AnimalAction, AnimalInputSignal};

/// Типаж, определяющий мозг животного.
pub trait AnimalBrain : Default {
    /// Действие агента (основной метод, определяющий поведение агента).
    fn action(&mut self, inputs: &AnimalInputSignal) -> AnimalAction;

    /// Клонирует мозг агента (со случайными мутациями).
    fn clone_with_mutation(& self) -> Self;
}