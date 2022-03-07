# Backend Assessment

Backend assessment assignment implemented using Rust + Rocket.rs + SQLite. Uses JWT tokens for authorization.

# API
## /register

- Request Body => JSON
    ```
    {
        "name" : String
        "email" : String
        "password" : String
        "account_type" : "Doctor" | "Patient" | "Admin"
    }
    ```

## /login
- Request Body => JSON
    ```
    {
        "email": String,
        "password": String
    }
    ```
- Response Body => JWT Token

## /doctors

- Response Body => JSON
    ```
    [DoctorID]
    ```



## /doctors/<doctor_id>
- Path Params
    ```
    doctor_id: Integer
    ```


## /doctors/<doctor_id>/slots
- Path Params
    ```
    doctor_id: Integer
    ```
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Request Body => JSON
    ```
    {
        "date": ISO-8601 String
    }
    ```
- Response Body => JSON
    ```
    {
        "patient_id": Option<Integer>, //Admins and Doctors only
        "start_date": ISO-8601 String,
        "duration": Integer
    }
    ```
## /doctors/<doctor_id>/book

Books an appointment with a doctor. Must be a Patient.
- Path Params
    ```
    doctor_id: Integer
    ```
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Request Body => JSON
    ```
    {
        "start_date": ISO-8601 String
        "duration": Integer
    }
    ```
- Response Body => Appointment ID Integer

## /appointments/<appointment_id>/cancel

Cancels the appointment. Must be Doctor or Admin

- Path Params
    ```
    appointment_id: Integer
    ```
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
## /doctors/available

Shows available doctors for a given date. Must be an Admin.
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Request Body => JSON
    ```
    {
        "date": ISO-8601 String
    }
    ```
- Response Body => JSON
    ```
    [DoctorID]
    ```
## /appointments/<appointment_id>

Shows appointment detials if authorized.
- Path Params
    ```
    appointment_id: Integer
    ```
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Response Body => JSON
  ```
    {
        "id": Integer,
        "doctor_id": Integer,
        "patient_id": Integer,
        "start_date": DateTime<FixedOffset>,
        "duration": Integer,
        "status": AppointmentStatus,
    }
  ```
## /patients/<patient_id>/history
- Path Params
    ```
    patient_id: Integer
    ```
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Response Body => JSON
  ```
  [
    {
        "id": Integer,
        "doctor_id": Integer,
        "patient_id": Integer,
        "start_date": DateTime<FixedOffset>,
        "duration": Integer,
        "status": AppointmentStatus,
    }
  ]
  ```
## /doctors/by_top_appointments

Lists doctors with the most appointments in a given day. Admin Only.

- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Request Body => JSON
    ```
    {
        "date": ISO-8601 String
    }
    ```
- Response Body => JSON
  ```
  {
    "doctor_id": Integer,
    "appointments_count": Integer,
    "booked_mins": Integer,
  }
  ```
## /doctors/with_six_hours_plus
Lists doctors with the 6+ hours of appointments in a given day. Admin Only.
- Request Header
    ```
    Authorization: Bearer <JWT Token>
    ```
- Request Body => JSON
    ```
    {
        "date": ISO-8601 String
    }
    ```
- Response Body => JSON
  ```
  {
    "doctor_id": Integer,
    "appointments_count": Integer,
    "booked_mins": Integer,
  }
  ```