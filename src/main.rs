#[macro_use]extern crate rocket;
use rocket::Request;
use rocket::{State, Shutdown};
use rocket::serde::{Serialize, Deserialize};

use rocket::fs::{relative, FileServer};
use rocket::form::Form;
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;

use rocket_auth::{Users, Error, Auth, Signup, Login};


#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let _res = queue.send(form.into_inner());
}

#[get("/say/<say>")]
async fn say(say: &str) -> String {
    say.to_string()
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[post("/signup", data="<form>")]
async fn signup(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into());
    Ok("You signed up.")
}

#[post("/login", data="<form>")]
async fn login(form: rocket::serde::json::Json<Login>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.login(&form).await?;
    Ok("You're logged in.")
}

#[get("/logout")]
fn logout(auth: Auth<'_>) {
    auth.logout();
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![say, post, events, signup , login, logout])
        .mount("/", FileServer::from(relative!("static")))
        .manage(users)
        .register("/", catchers![not_found])
        .launch()
        .await?;

    Ok(())
}