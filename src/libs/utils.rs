pub fn print_type_of<T>(_: &T) {
  println!("{}", unsafe { std::intrinsics::type_name::<T>() });
}
