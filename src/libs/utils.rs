extern crate mac_address;
extern crate uuid;

pub fn print_type_of<T>(_: &T) {
  println!("{}", unsafe { std::intrinsics::type_name::<T>() });
}

pub fn generate_uuid() -> String {
  uuid::Uuid::new_v4().to_hyphenated().to_string()
}

pub fn get_unique_name() -> Result<String, mac_address::MacAddressError> {
  #[cfg(target_os = "linux")]
  let name = "eth0";

  #[cfg(target_os = "macos")]
  let name = "en0";

  #[cfg(target_os = "windows")]
  let name = "Ethernet";

  match mac_address::mac_address_by_name(name) {
    Ok(Some(ma)) => Ok(ma.to_string()),
    Ok(None) => Ok(generate_uuid()),
    Err(e) => Err(e),
  }
}
