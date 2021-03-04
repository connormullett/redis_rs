mod connection;

#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;

#[cfg(test)]
mod tests {
    use crate::connection;

    #[test]
    fn test_parse_command() {
        let command = String::from("GET FOO");

        let parsed_command = connection::parse_command(&command).unwrap();

        assert_eq!("*2\r\n$3\r\nGET\r\n$3\r\nFOO\r\n", parsed_command);
    }
}
