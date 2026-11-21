fn main() {
    let texto = String::from("Olá");
    {
        let referencia = &texto;
        println!("{referencia}");
    }
    println!("{texto}");
}
