#El model ja esta dins
#Si volem fer fine tunning amb Lora i carregar un propi

let blob = std::fs::read("my_needle.cact")?;
Needle::load_weights(&blob)?;   // sustituye los pesos horneados por los tuyos
let needle = Needle::init("", &tools_json)?;  // ahora usa los pesos que acabas de cargar
