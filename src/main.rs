  #[cfg(not(target_arch = "wasm32"))]
  fn main() {
      let args: Vec<String> = std::env::args().collect();
      if args.len() > 1 {
          corridor::run_native_with_graph(&args[1]);
      } else {
          corridor::run_native();
      }
  }
