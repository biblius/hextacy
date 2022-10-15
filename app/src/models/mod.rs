pub mod role;
pub mod schema;
pub mod session;
pub mod user;

#[cfg(test)]
mod tests {
    use infrastructure::{
        config::env,
        storage::postgres::Pg,
        utility::rand::{self, Rng},
    };

    static ALPHABET: [char; 52] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    use crate::models::role::Role;

    use super::user::User;

    #[test]
    fn test_users() {
        env::load_from_file("../.env").unwrap();
        user_insert();
    }

    fn user_insert() {
        let mut pool = Pg::new().connect().unwrap();
        let mut rng = rand::thread_rng();
        let (name, domain) = {
            let mut s = String::new();
            let mut v = String::new();
            for _ in 0..5 {
                s.push(ALPHABET[rng.gen_range(0..52)]);
                v.push(ALPHABET[rng.gen_range(0..52)]);
            }
            (s, v)
        };
        let email = format!("{}@{}.com", name, domain);
        let user = User::create_initial(&email, "bebliuskhan", &mut pool).unwrap();
        assert_eq!(user.email, email);
        assert!(matches!(user.role, Role::User))
    }
}
