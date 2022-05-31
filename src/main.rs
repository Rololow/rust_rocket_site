#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/name/<name>")]
async fn name(name: &str) -> String {
    format!("your name is {}", name)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index, name])
        .launch()
        .await?;

    Ok(())
}
