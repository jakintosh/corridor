  #[cfg(not(target_arch = "wasm32"))]
  fn main() {
      corridor::run_native();
  }
