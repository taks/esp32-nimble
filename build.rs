fn main() {
  embuild::espidf::sysenv::output();

  println!("cargo::rustc-check-cfg=cfg(esp32)");
  println!("cargo::rustc-check-cfg=cfg(esp32c3)");

  println!("cargo::rustc-check-cfg=cfg(esp_idf_soc_esp_nimble_controller)");
  println!("cargo::rustc-check-cfg=cfg(esp_idf_bt_nimble_ext_adv)");

  println!("cargo::rustc-check-cfg=cfg(esp_idf_version_major, values(\"4\", \"5\"))");
  println!("cargo::rustc-check-cfg=cfg(esp_idf_version_minor, values(\"2\"))");
  println!("cargo::rustc-check-cfg=cfg(esp_idf_version_patch, values(\"0\"))");
  println!("cargo::rustc-check-cfg=cfg(esp_idf_version_patch, values(\"1\"))");
  println!("cargo::rustc-check-cfg=cfg(esp_idf_version_patch, values(\"2\"))");

  let esp_idf_info = embuild::espidf::sysenv::cfg_args().unwrap();
  let version: Vec<usize> = ["major", "minor", "patch"]
    .iter()
    .filter_map(|ver_part| {
      let search_string = format!("esp_idf_version_{}", ver_part);
      esp_idf_info.args.iter().find_map(|arg| {
        if arg.starts_with(&search_string) {
          let version_str = arg.split_terminator('"').nth(1).unwrap();
          Some(version_str.parse::<usize>().unwrap())
        } else {
          None
        }
      })
    })
    .collect();

  println!("cargo::rustc-check-cfg=cfg(cpfd)");
  if version > vec![5, 2, 0] {
    println!("cargo::rustc-cfg=cpfd");
  }
}
