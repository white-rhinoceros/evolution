//! "Простой мозг" животного.

extern crate ndarray;

use crate::animal::brains::AnimalBrain;
use crate::animal::{AnimalAction, AnimalInputSignal};
use ndarray::prelude::*;

type UseType = u8;

/// Константа, определяющая размер "вектора" входных сигналов.
const INPUT_VECTOR_SIZE: usize = 12;

/// Константа, определяющая размер "вектора" выходных сигналов (по числу возможных действий).
const OUTPUT_VECTOR_SIZE: usize = 4;

/// Структура, реализующая мозг агента.
pub struct Brain {
    inputs: Array1<UseType>,   // Входной вектор сигналов.
    weights: Array2<UseType>,  // Матрица весов.
    bias: Array1<UseType>,     // Вектор смещений.
    actions: Array1<UseType>,  // Вектор выходных сигналов.
}

impl Brain {
    fn get_weight() -> UseType {
        todo!()
    }
}

impl AnimalBrain for Brain {
    /// Конструктор.
    fn new() -> Brain {
        let inputs = Array::<UseType, _>::zeros(INPUT_VECTOR_SIZE.f());
        let weights = Array::<UseType, _>::zeros((INPUT_VECTOR_SIZE, OUTPUT_VECTOR_SIZE).f());
        let bias = Array::<UseType, _>::zeros(OUTPUT_VECTOR_SIZE.f());
        let actions = Array::<UseType, _>::zeros(OUTPUT_VECTOR_SIZE.f());

        Brain {
            inputs,
            weights,
            bias,
            actions,
        }
    }

    /// Действие агента.
    fn action(&mut self, _inputs: &AnimalInputSignal) -> AnimalAction {
        // /* Forward propagate the inputs through the neural network */
        // for ( out = 0 ; out < MAX_OUTPUTS ; out++ ) {
        //
        //     /* Initialize the output node with the bias */
        //     agent->actions[out] = agent->biaso[out];
        //
        //     /* Multiply the inputs by the weights for this output node */
        //     for ( in = 0 ; in < MAX_INPUTS ; in++ ) {
        //
        //         agent->actions[out] +=
        //             ( agent->inputs[in] * agent->weight_oi[(out * MAX_INPUTS)+in] );
        //
        //     }
        //
        // }
        //
        // largest = -9;
        // winner = -1;
        //
        // /* Select the largest node (winner-takes-all network) */
        // for ( out = 0 ; out < MAX_OUTPUTS ; out++ ) {
        //     if (agent->actions[out] >= largest) {
        //         largest = agent->actions[out];
        //         winner = out;
        //     }
        // }



        AnimalAction::Move
    }

    fn clone_with_mutation(&self) -> Self {
        todo!()
    }
}