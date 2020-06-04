macro_rules! with_base_url {
  ($($arg:tt)*) => ({
      #[cfg(test)]
      use mockito;

      #[cfg(test)]
      let url = format!("{}/{}", &mockito::server_url(), format!($($arg)*));
      #[cfg(not(test))]
      let url = format!("{}/{}", BASE_URL, format!($($arg)*));

      url
  })
}

pub mod basic_info;
pub mod repos;
pub mod secrets;
pub mod workflows;
