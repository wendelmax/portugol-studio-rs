fn main() {
    let texto = String::from("Olá");
    let ref1 = &texto;
    let ref2 = &texto;
    println!("{ref1} {ref2}");
    println!("{texto}");
}
