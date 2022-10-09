pub async fn handler() {
    /*     auth_form: web::Form<AuthForm>,
        state: web::Data<AppState>,
    ) -> Result<HttpResponse, GlobalError> {
        info!("{}{:?}", "User login : ".cyan(), auth_form);
        let db_connection = db_pool::connect(&state)?;
        if let Some(user) = User::find_by_uname(&db_connection, &auth_form.username)? {
            let verified = bcrypt::verify(auth_form.password.clone(), &user.password)?;
            if !verified {
                return Err(GlobalError::AuthenticationError(
                    AuthenticationError::BadPassword,
                ));
            }
            AuthResponse::succeed_with_token(user)
        } else {
            Err(GlobalError::AuthenticationError(
                AuthenticationError::UserNotFound,
            ))
        } */
}
