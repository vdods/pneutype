fn pneu_str_ref_taking_fn<P: std::fmt::Display + pneutype::PneuStr + ?Sized>(p: &P) {
    println!("p: {}", p);
}

#[test]
fn test_str_impls_pneu_str() {
    let s = "HIPPO";
    pneu_str_ref_taking_fn(s);
}

fn pneu_string_ref_taking_fn<P: std::fmt::Display + pneutype::PneuString + ?Sized>(p: &P) {
    println!("p: {}", p);
}

fn pneu_string_taking_fn<P: std::fmt::Display + pneutype::PneuString + ?Sized>(p: P) {
    println!("p: {}", p);
}

#[test]
fn test_string_impls_pneu_str() {
    let s = "HIPPO".to_string();
    pneu_string_ref_taking_fn(&s);
    pneu_string_taking_fn(s);
}
