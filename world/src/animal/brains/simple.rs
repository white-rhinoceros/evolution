//! "Простой мозг" животного.
//! TODO: Попробовать расширить нейронку до 5 выходных значений (+ None).

extern crate nalgebra;
use nalgebra::{SVector, SMatrix};


use crate::animal::brains::AnimalBrain;
use crate::animal::{AnimalAction, AnimalInputSignal};

use rand::Rng;

type WeightType = f32;

/// Константа, определяющая размер "вектора" входных сигналов.
const INPUT_VECTOR_SIZE: usize = 12;

/// Константа, определяющая размер "вектора" выходных сигналов (по числу возможных действий).
const OUTPUT_VECTOR_SIZE: usize = 4;

/// Структура, реализующая мозг агента.
pub struct Brain {
    // Матрица весов.
    weights: SMatrix::<WeightType, OUTPUT_VECTOR_SIZE, INPUT_VECTOR_SIZE>,
    // Вектор смещений.
    bias: SVector::<WeightType, OUTPUT_VECTOR_SIZE>,
}

impl Brain {
    /// Генерация случайного веса для нейросети.
    /// Результат принадлежит диапазону [-1, 1].
    fn generate_weight() -> WeightType {
        rand::thread_rng().gen_range(-1.0..=1.0)
    }
}

impl Default for Brain {
    /// Мозг по умолчанию (заполняется случайными значениями).
    fn default() -> Self {
        Brain {
            weights: SMatrix::new_random(),
            bias: SVector::new_random(),
        }
    }
}

impl AnimalBrain for Brain {
    /// Действие агента.
    fn action(&mut self, percept: &AnimalInputSignal) -> AnimalAction {

        let mut inputs = SVector::<WeightType, INPUT_VECTOR_SIZE>::zeros();
        // Конвертируем восприятие животного во входной вектор.
        inputs[0]  = percept.plant_front as WeightType;
        inputs[1]  = percept.plant_left as WeightType;
        inputs[2]  = percept.plant_right as WeightType;
        inputs[3]  = percept.plant_proximity as WeightType;

        inputs[4]  = percept.herbivore_front as WeightType;
        inputs[5]  = percept.herbivore_left as WeightType;
        inputs[6]  = percept.herbivore_right as WeightType;
        inputs[7]  = percept.herbivore_proximity as WeightType;

        inputs[8]  = percept.carnivore_front as WeightType;
        inputs[9]  = percept.carnivore_left as WeightType;
        inputs[10] = percept.carnivore_right as WeightType;
        inputs[11] = percept.carnivore_proximity as WeightType;

        // Подсчитаем выходные значения.
        let actions = self.bias + self.weights * inputs;
        let mut largest: WeightType = Default::default();
        let mut out: usize = 0;

        // Select the largest node (winner-takes-all network).
        for (index, action) in actions.iter().enumerate() {
            if *action > largest {
                largest = *action;
                out = index;
            }
        }

        match out {
            0 => AnimalAction::TurnLeft,
            1 => AnimalAction::TurnRight,
            2 => AnimalAction::Move,
            3 => AnimalAction::Eat,
            _ => AnimalAction::None,
        }
    }

    /// Клонировать мозг с мутацией одного веса. Вес выбирается случайно,
    /// как и значение.
    fn clone_with_mutation(&self) -> Self {
        todo!()
    }
}