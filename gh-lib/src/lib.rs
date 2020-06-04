#[cfg(not(test))]
pub const BASE_URL: &str = "https://api.github.com";

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

pub mod core;
mod graphql;
mod utils;
