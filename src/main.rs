use gloo_render::{request_animation_frame, AnimationFrame};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{
    HtmlCanvasElement,
    HtmlImageElement,
    WebGl2RenderingContext as GL,
    WebGlTexture,
};
use yew::{html::Scope, html, Component, Context, Html, NodeRef};
use std::rc::Rc;

pub enum Msg {
    Render(f64),
}

pub struct App {
    gl: Option<GL>,
    node_ref: NodeRef,
    texture: Option<Rc<WebGlTexture>>,
    _render_loop: Option<AnimationFrame>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            gl: None,
            node_ref: NodeRef::default(),
            _render_loop: None,
            texture: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render(timestamp) => {
                // TODO Render functions are likely to get quite large, so it is good practice to split
                // it into it's own function rather than keeping it inline in the update match
                // case. This also allows for updating other UI elements that may be rendered in
                // the DOM like a framerate counter, or other overlaid textual elements.
                self.render_gl(timestamp, ctx.link());
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas ref={self.node_ref.clone()} />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        // TODO Once rendered, store references for the canvas and GL context. These can be used for
        // resizing the rendering area when the window or canvas element are resized, as well as
        // for making GL calls.

        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();

        let gl: GL = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        self.gl = Some(gl);

        // TODO In a more complex use-case, there will be additional WebGL initialization that should be
        // done here, such as enabling or disabling depth testing, depth functions, face
        // culling etc.

        if first_render {
            self.load_texture("img/image.png");
            // The callback to request animation frame is passed a time value which can be used for
            // rendering motion independent of the framerate which may vary.
            let handle = {
                let link = ctx.link().clone();
                request_animation_frame(move |time| link.send_message(Msg::Render(time)))
            };

            // A reference to the handle must be stored, otherwise it is dropped and the render won't
            // occur.
            self._render_loop = Some(handle);
        }
    }
}

impl App {
    fn render_gl(&mut self, timestamp: f64, link: &Scope<Self>) {
        if self.texture.is_none() {
            return;
        }

        let gl = self.gl.as_ref().expect("GL Context not initialized!");

        let vert_code = include_str!("./basic.vert");
        let frag_code = include_str!("./basic.frag");

        let vertices: Vec<f32> = vec![
            -0.5, 0.5,
                0.5, 0.5,
                -0.5, -0.5,
            -0.5, -0.5,
                0.5, 0.5,
                0.5, -0.5,
        ];
        let vertex_buffer = gl.create_buffer().unwrap();
        let verts = js_sys::Float32Array::from(vertices.as_slice());

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vertex_buffer));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);

        let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
        gl.shader_source(&vert_shader, vert_code);
        gl.compile_shader(&vert_shader);

        let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
        gl.shader_source(&frag_shader, frag_code);
        gl.compile_shader(&frag_shader);

        let shader_program = gl.create_program().unwrap();
        gl.attach_shader(&shader_program, &vert_shader);
        gl.attach_shader(&shader_program, &frag_shader);
        gl.link_program(&shader_program);

        gl.use_program(Some(&shader_program));

        // Attach the position vector as an attribute for the GL context.
        let position = gl.get_attrib_location(&shader_program, "a_position") as u32;
        gl.vertex_attrib_pointer_with_i32(position, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(position);

        // Setup the textCoord
        let tex_coord = gl.get_attrib_location(&shader_program, "a_texCoord") as u32;
        let tex_coord_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&tex_coord_buffer));
        let vertices: Vec<f32> = vec![
            0.0,  0.0,
            1.0,  0.0,
            0.0,  1.0,
            0.0,  1.0,
            1.0,  0.0,
            1.0,  1.0
        ];
        let verts = js_sys::Float32Array::from(vertices.as_slice());
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &verts, GL::STATIC_DRAW);
        gl.vertex_attrib_pointer_with_i32(tex_coord, 2, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(tex_coord);

        // Attach the time as a uniform for the GL context.
        let time = gl.get_uniform_location(&shader_program, "u_time");
        gl.uniform1f(time.as_ref(), timestamp as f32);

        gl.draw_arrays(GL::TRIANGLES, 0, 6);

        let handle = {
            let link = link.clone();
            request_animation_frame(move |time| link.send_message(Msg::Render(time)))
        };

        // A reference to the new handle must be retained for the next render to run.
        self._render_loop = Some(handle);
    }

    fn load_texture(&mut self, img_src: &str) {
        let gl = self.gl.as_ref().expect("GL Context not initialized!");

        let texture = gl.create_texture().expect("Cannot create gl texture");
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        let level = 0;
        let internal_format = GL::RGBA;
        let width = 1;
        let height = 1;
        let border = 0;
        let src_format = GL::RGBA;
        let src_type = GL::UNSIGNED_BYTE;

        // Now upload single pixel.
        let pixel: [u8; 4] = [0, 0, 255, 255];
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            level,
            internal_format as i32,
            width,
            height,
            border,
            src_format,
            src_type,
            Some(&pixel),
        ).unwrap();

        let img = HtmlImageElement::new().unwrap();
        img.set_cross_origin(Some(""));

        let imgrc = Rc::new(img);

        let texture = Rc::new(texture);

        {
            let img = imgrc.clone();
            let texture = texture.clone();
            let gl = Rc::new(gl.clone());
            let on_load = Closure::wrap(Box::new(move || {
                gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

                if let Err(e) = gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                    GL::TEXTURE_2D,
                    level,
                    internal_format as i32,
                    src_format,
                    src_type,
                    &img,
                ) {
                    // TODO better error handling...
                    //console::log_1(&e);
                    return;
                }

                gl.generate_mipmap(GL::TEXTURE_2D);
            }) as Box<dyn FnMut()>);
            imgrc.set_onload(Some(on_load.as_ref().unchecked_ref()));

           // Normally we'd store the handle to later get dropped at an appropriate
            // time but for now we want it to be a global handler so we use the
            // forget method to drop it without invalidating the closure. Note that
            // this is leaking memory in Rust, so this should be done judiciously!
            on_load.forget();
        }

        imgrc.set_src(img_src);

        self.texture = Some(texture);
    }
}

fn main() {
    yew::start_app::<App>();
}
