//! "Простой мозг" животного.

extern crate nalgebra;
use nalgebra::{SVector, SMatrix};
use crate::animal::brains::AnimalBrain;
use crate::animal::{AnimalAction, AnimalInputSignal, MAX_ACTIONS};
use rand::Rng;

type WeightType = f32;

/// Константа, определяющая размер "вектора" входных сигналов.
const INPUT_VECTOR_SIZE: usize = 12;

/// Константа, определяющая размер "вектора" выходных сигналов (по числу возможных действий).
const OUTPUT_VECTOR_SIZE: usize = 4;

/// Генерация случайного веса для нейросети.
/// Результат принадлежит диапазону [-1, 1].
fn generate_weight() -> WeightType {
    rand::thread_rng().gen_range(-1.0..=1.0)
}

/// Структура, реализующая мозг агента.
pub struct Brain {
    // Матрица весов.
    weights: SMatrix::<WeightType, OUTPUT_VECTOR_SIZE, INPUT_VECTOR_SIZE>,
    // Вектор смещений.
    bias: SVector::<WeightType, OUTPUT_VECTOR_SIZE>,
}

impl Brain {
    fn choose_action(&self, actions: SVector::<WeightType, OUTPUT_VECTOR_SIZE>) -> AnimalAction {
        // Определяем действие - победитель.
        // Применим функцию активации к выходным нейронам и получим распределение
        // активированных нейронов.
        let mut ranges: Vec<WeightType> = Vec::with_capacity(MAX_ACTIONS);
        let mut outs: Vec<usize> = Vec::with_capacity(MAX_ACTIONS);
        let mut total: WeightType = 0 as WeightType;

        for (index, action) in actions.iter().enumerate() {
            if *action > 0 as WeightType {
                outs.push(index);
                ranges.push(*action);
                total += *action;
            }
        }

        // Активированных нейронов нет.
        if ranges.is_empty() {
            return AnimalAction::None;
        }

        // Получаем случайное значение в диапазоне суммы всех выходных значений.
        let choose: WeightType = rand::thread_rng().gen_range(0.0..=total);

        // Разыгрываем случайную величину, в соответствии с распределением активированных
        // нейронов.
        let mut x1: WeightType = 0 as WeightType;
        let mut x2: WeightType = 0 as WeightType;

        for (i, v) in ranges.iter().enumerate() {
            x2 += v;
            if choose >= x1 && choose < x2 {
                return match outs[i] {
                    0 => AnimalAction::TurnLeft,
                    1 => AnimalAction::TurnRight,
                    2 => AnimalAction::Move,
                    3 => AnimalAction::Eat,
                    _ => AnimalAction::None,
                };
            };
            x1 += v;
        }

        panic!("Алгоритм выбора действия для животного сработал некорректно. Достигнута точка, \
                которую мы ну ни как достигнуть не могли.");
    }

    // fn choose_largest(&self, actions: SVector::<WeightType, OUTPUT_VECTOR_SIZE>) -> AnimalAction {
    //     let mut largest: WeightType = Default::default();
    //     let mut out: usize = 0;
    //
    //     // Select the largest node (winner-takes-all network).
    //     for (index, action) in actions.iter().enumerate() {
    //         if *action > largest {
    //             largest = *action;
    //             out = index;
    //         }
    //     }
    //
    //     match out {
    //         0 => AnimalAction::TurnLeft,
    //         1 => AnimalAction::TurnRight,
    //         2 => AnimalAction::Move,
    //         3 => AnimalAction::Eat,
    //         _ => AnimalAction::None,
    //     }
    // }
}

impl Default for Brain {
    /// Мозг по умолчанию (заполняется случайными значениями).
    fn default() -> Self {
        let mut weights = SMatrix::<WeightType, OUTPUT_VECTOR_SIZE, INPUT_VECTOR_SIZE>::zeros();
        for i in 0..OUTPUT_VECTOR_SIZE * INPUT_VECTOR_SIZE {
            weights[i] = generate_weight();
        }

        let mut bias = SVector::<WeightType, OUTPUT_VECTOR_SIZE>::zeros();
        for i in 0..OUTPUT_VECTOR_SIZE {
            bias[i] = generate_weight();
        }

        Brain {
            weights,
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
        let actions: SVector::<WeightType, OUTPUT_VECTOR_SIZE>  = self.bias + self.weights * inputs;
        // Передаем владение actions.
        self.choose_action(actions)
    }

    /// Клонировать мозг с мутацией одного веса. Вес выбирается случайно,
    /// как и значение.
    fn clone_with_mutation(&self) -> Self {
        todo!()
    }

}