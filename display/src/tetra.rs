use std::sync::mpsc::Receiver;
use crate::{CellStuff, Map};

use tetra::graphics::{self, Color, Texture};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};
use tetra::error::Result as TetraResult;

const MAX_WIDTH_SIZE: usize = 1920;

const MAX_HEIGHT_SIZE: usize = 1080;

const CARNIVORE_NAME: &str = "wolf";

const HERBIVORE_NAME: &str = "sheep";

const ANIMAL_DIRECTIONS: [&str; 4] = ["left", "right", "front", "back"];

const BACKGROUND_COLOR:Color = Color::rgb(0.392, 0.584, 0.929);

/// Возможные варианты размера текстур.
#[derive(Copy, Clone)]
enum TextureSize {
    Size63 = 63,
    Size40 = 40,
    Size20 = 20,
}

use self::TextureSize::*;
const TEXTURE_SIZES:[TextureSize; 3] = [Size63, Size40, Size20];

pub struct Window {
    // Канал для получения данных о состоянии мира.
    receiver: Receiver<Map>,

    // Путь до файлов с изображениями текстур.
    asset_path: String,

    // Размер текстур.
    texture_size: TextureSize,

    // Поля, для хранения текстур.
    killed_animal_texture: Texture,
    dead_animal_texture: Texture,
    herbivore_texture: Vec<Texture>,
    carnivore_texture: Vec<Texture>,
    plant_texture: Texture,

    map: Map,
}

impl Window {
    /// Создает новый экземпляр драйвера.
    ///
    /// # Arguments
    ///
    /// * `width`: Шрина мира.
    /// * `height`: Высота мира.
    /// * `receiver`: Канал для получения данных.
    /// * `asset_path`: Путь к файлам изображений.
    /// * `title`: Заглавие окна программы.
    ///
    /// returns: Result<(), String>
    pub(crate) fn new(
        width: usize,
        height: usize,
        receiver: Receiver<Map>,
        base_path: &str,
        title: &str
    ) -> Result<(), String> {
        let sizes = Self::get_window_size(width, height)?;

        // Создаем контекст
        let mut ctx = ContextBuilder::new(title, sizes.0, sizes.1)
            .high_dpi(true)
            .show_mouse(true)
            .quit_on_escape(true)
            .build()
            .expect("Создание контекста тетра пало");

        let mut asset_path = base_path.to_owned();
        asset_path.push_str("/resources/");

        ctx.run(move |ctx| {
            let killed_animal_texture = Self::load_texture(
                ctx, &asset_path, sizes.2, "blood"
            )?;

            let dead_animal_texture = Self::load_texture(
                ctx, &asset_path, sizes.2, "ghost"
            )?;

            let plant_texture = Self::load_texture(
                ctx, &asset_path, sizes.2, "plant"
            )?;

            let herbivore_texture = Self::load_animal_texture(
                ctx,  &asset_path, sizes.2, HERBIVORE_NAME
            )?;

            let carnivore_texture = Self::load_animal_texture(
                ctx,  &asset_path, sizes.2, CARNIVORE_NAME
            )?;

            Ok(Window {
                receiver,
                asset_path,
                texture_size: sizes.2,
                killed_animal_texture,
                dead_animal_texture,
                herbivore_texture,
                carnivore_texture,
                plant_texture,
                map: vec![],
            })
        }).expect("Тетра пала!");

        Ok(())
    }

    /// Возвращает актуальные размеры окна и тексур для данного размера мира.
    ///
    /// # Arguments
    ///
    /// * `width`: Шрина мира.
    /// * `height`: Высота мира.
    ///
    /// returns: Result<(i32, i32, TextureSize), String>
    fn get_window_size(width: usize,  height: usize) -> Result<(i32, i32, TextureSize), String> {
        for size in TEXTURE_SIZES {
            let window_with = width * size as usize;
            let window_height = height * size as usize;

            if window_with <= MAX_WIDTH_SIZE && window_height <= MAX_HEIGHT_SIZE {
                return Ok((window_with as i32, window_height as i32, size));
            }
        }

        Err("Мир слишком велик ".to_string())
    }

    /// Загружает текстуру из ресурсов.
    ///
    /// # Arguments
    ///
    /// * `ctx`: Контекст tetra.
    /// * `asset_path`: Путь к изображениям текстур.
    /// * `texture_size`: Размер загружаемых текстур.
    /// * `target`: Имя загружаемого объекта.
    ///
    /// returns: Result<Texture, TetraError>
    fn load_texture(
        ctx: &mut Context,
        asset_path: &String,
        texture_size: TextureSize,
        target: &str
    ) -> TetraResult<Texture> {
        let mut path = asset_path.clone();

        path.push_str(target);
        path.push('/');
        path.push_str((texture_size as usize).to_string().as_str());
        path.push_str(".png");

        Texture::new(ctx, path)
    }

    /// Загружает текстуры животного соотвествующие четырем направлениям
    /// движения.
    ///
    /// # Arguments
    ///
    /// * `ctx`: Контекст tetra.
    /// * `asset_path`: Путь к изображениям текстур.
    /// * `texture_size`: Размер загружаемых текстур.
    /// * `target`: Имя загружаемого объекта.
    ///
    /// returns: Result<Texture, TetraError>
    fn load_animal_texture(
        ctx: &mut Context,
        asset_path: &String,
        texture_size: TextureSize,
        target: &str
    ) -> TetraResult<Vec<Texture>> {
        let mut tetxtures = Vec::with_capacity(4);

        for direct in ANIMAL_DIRECTIONS {
            let mut path = asset_path.clone();

            path.push_str(target);
            path.push('/');
            path.push_str(direct);
            path.push('_');
            path.push_str((texture_size as usize).to_string().as_str());
            path.push_str(".png");

            match Texture::new(ctx, path) {
                Ok(t) => {
                    tetxtures.push(t);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(tetxtures)
    }

    /// Преобразует координаты мира в экранные координаты.
    ///
    /// # Arguments
    ///
    /// * `width`:
    /// * `height`:
    ///
    /// returns: Vec2<f32>
    fn get_window_coords(&self, width: usize,  height: usize) -> Vec2<f32> {
        let width = (width * (self.texture_size as usize)) as f32;
        let height = (height * (self.texture_size as usize)) as f32;

        Vec2::new(width, height)
    }
}

impl State for Window {
    /// Обрабатывает ввод данных от пользователя (клавиатура, мыщ, и т.д.)
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.receiver.try_recv() {
            Ok(map) => {
                self.map = map;
            }
            Err(_) => {
                // В канал не передали данные.
            }
        }

        Ok(())
    }

    /// Отображает мир.
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, BACKGROUND_COLOR);

        for p in &self.map {
            match p.2 {
                CellStuff::KilledAnimal => {
                    self.killed_animal_texture.draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::DeadAnimal => {
                    self.dead_animal_texture.draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::HerbLeft => {
                    self.herbivore_texture[0].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::HerbRight => {
                    self.herbivore_texture[1].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::HerbFront => {
                    self.herbivore_texture[2].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::HerbBack => {
                    self.herbivore_texture[3].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::CarnLeft => {
                    self.carnivore_texture[0].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::CarnRight => {
                    self.carnivore_texture[1].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::CarnFront => {
                    self.carnivore_texture[2].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::CarnBack => {
                    self.carnivore_texture[3].draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::Plant => {
                    self.plant_texture.draw(ctx, self.get_window_coords(p.0, p.1));
                }
                CellStuff::None => {}
            }
        }

        Ok(())
    }
}


