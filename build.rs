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
}
