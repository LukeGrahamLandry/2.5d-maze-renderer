use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(a: &str);
}

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => {{
        #[cfg(target_arch = "wasm32")]
        { ($crate::log::log(&format_args!($($t)*).to_string())) }
        #[cfg(not(target_arch = "wasm32"))]
        { std::println!($($t)*) }
    }}
}
