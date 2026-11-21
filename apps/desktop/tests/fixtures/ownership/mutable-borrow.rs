fn main() {
    let mut texto = String::from("Olá");
    let referencia = &mut texto;
    referencia.push_str("!");
    println!("{referencia}");
    println!("{texto}");
}
