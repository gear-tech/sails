use sails_rs::{export, service};

pub(super) struct MyService;

#[service]
impl MyService {
    #[export(unwrap_result)]
    pub async fn do_this(&mut self, p1: u32, p2: String) -> Result<String, String> {
        Ok(format!("{p1}: ") + &p2)
    }

    #[export(route = "Parse", unwrap_result)]
    pub async fn parse_result(&mut self, s: String) -> Result<u32, String> {
        let res = str::parse::<u32>(s.as_str()).map_err(|_| format!("failed to parse `{}`", s))?;
        Ok(res)
    }

    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}
