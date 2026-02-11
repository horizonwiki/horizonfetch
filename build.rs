    fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("src/horizonfetch.ico");
    res.compile().unwrap();
    }