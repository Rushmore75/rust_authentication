use rocket::{routes, fs::FileServer, serde::json::{Json, serde_json::json, Value}, get};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let _rocket = rocket::build()
        .mount("/", FileServer::from("www/"))
        .mount("/", routes![example])
        .launch()
        .await?;
    Ok(())
}

#[get("/test")]
fn example() -> Json<Value> {
    let x = json!({
            "id": 20,
            "status": "Active",
            "type": "AIR CONDITIONING",
            "category": "",
            "subcategory": "",
            "item": "Orifice Tube",
            "description": "Orifice Tube",
            "descriptionfull": "Orifice Tube  38623",
            "qoh": 18,
            "cost": 1.32,
            "price": 11.06
        });
    
    Json(x)
}