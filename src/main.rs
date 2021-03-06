#[macro_use]
extern crate rocket;
extern crate argon2;
extern crate derive_more;

mod api;
mod db;

#[rocket::main]
async fn main() -> Result<(), rocket::error::Error> {
    {
        let db = db::DB::default().unwrap();
        db.init_schema().unwrap();
    }

    rocket::build()
        .mount(
            "/",
            routes![
                api::register,
                api::login,
                api::doctors,
                api::doctor_info,
                api::doctor_booked_slots,
                api::book_doctor,
                api::cancel_appointment,
                api::available_doctors,
                api::appointment_details,
                api::patient_history,
                api::stats_top_appointments,
                api::stats_greaterthan_hours,
            ],
        )
        .launch()
        .await
}
