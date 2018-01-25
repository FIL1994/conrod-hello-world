#[cfg(all(feature="winit", feature="glium"))]
#[macro_use]
extern crate conrod;

mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod feature {
    extern crate find_folder;
    extern crate image;
    extern crate rand; // for making a random color.
    use conrod::{self, widget, Borderable, Colorable, Positionable, Sizeable, Widget, color};
    use conrod::backend::glium::glium::{self, Surface};
    use std;
    use support;

    struct Fonts {
        regular: conrod::text::font::Id,
        italic: conrod::text::font::Id,
        bold: conrod::text::font::Id,
    }

    pub fn main() {
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = ((WIDTH as f64) * 0.56) as u32;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("My Window Title")
            .with_dimensions(WIDTH, HEIGHT);
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        let noto_sans = assets.join("fonts/NotoSans");
        // Store our `font::Id`s in a struct for easy access in the `set_ui` function.
        let fonts = Fonts {
            regular: ui.fonts.insert_from_file(noto_sans.join("NotoSans-Regular.ttf")).unwrap(),
            italic: ui.fonts.insert_from_file(noto_sans.join("NotoSans-Italic.ttf")).unwrap(),
            bold: ui.fonts.insert_from_file(noto_sans.join("NotoSans-Bold.ttf")).unwrap(),
        };

        // Specify the default font to use when none is specified by the widget.
        //
        // By default, the theme's font_id field is `None`. In this case, the first font that is found
        // within the `Ui`'s `font::Map` will be used.
        ui.theme.font_id = Some(fonts.regular);

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // Declare the ID for each of our widgets.
        widget_ids!(
            struct Ids {
                 canvas,
                 button,
                 rust_logo,
                 master,
                 left_col,
                 middle_col,
                 right_col,
                 left_text,
                 middle_text,
                 right_text,
            }
        );
        let ids = Ids::new(ui.widget_id_generator());

        // Create our `conrod::image::Map` which describes each of our widget->image mappings.
        // In our case we only have one image, however the macro may be used to list multiple.
        let mut image_map = conrod::image::Map::new();

        struct ImageIds {
            normal: conrod::image::Id,
            hover: conrod::image::Id,
            press: conrod::image::Id,
        }

        // Load the images into our `ImageIds` type for easy access.
        let image_path = assets.join("images");
        let rust_logo = load_image(&display, image_path.join("rust.png"));
        let (w, h) = (rust_logo.get_width(), rust_logo.get_height().unwrap());
        let image_ids = ImageIds {
            normal: image_map.insert(rust_logo),
            hover: image_map.insert(load_image(&display, image_path.join("rust_hover.png"))),
            press: image_map.insert(load_image(&display, image_path.join("rust_press.png"))),
        };

        // We'll change the background colour with the image button.
        let mut bg_color = conrod::color::LIGHT_BLUE;

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::WindowEvent::Closed |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'main,
                        _ => (),
                    },
                    _ => (),
                }
            }

            {
                let ui = &mut ui.set_widgets();

                // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
                widget::Canvas::new()
                    .pad(30.0)
                    .color(bg_color)
                    .set(ids.canvas, ui);

                let my_button = widget::Button::image(image_ids.normal)
                    .hover_image(image_ids.hover)
                    .press_image(image_ids.press)
                    .w_h(w as conrod::Scalar, h as conrod::Scalar)
                    .middle_of(ids.canvas)
                    .border(0.0)
                    .set(ids.button, ui);

                // Button widget example button.
                if my_button.was_clicked() {
                    bg_color = color::rgb(rand::random(), rand::random(), rand::random());
                }

                const MY_TEXT: &'static str = "my text";
                widget::Text::new(MY_TEXT)
                    .font_id(fonts.regular)
                    .color(color::LIGHT_GREEN)
                    .mid_top()
                    .left_justify()
                    .line_spacing(10.0)
                    .set(ids.left_text, ui);
            }

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    // Load an image from our assets folder as a texture we can draw to the screen.
    fn load_image<P>(display: &glium::Display, path: P) -> glium::texture::SrgbTexture2d
        where P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(), image_dimensions);
        let texture = glium::texture::SrgbTexture2d::new(display, raw_image).unwrap();
        texture
    }
}

#[cfg(not(all(feature="winit", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` and `glium` features. \
                 Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
    }
}