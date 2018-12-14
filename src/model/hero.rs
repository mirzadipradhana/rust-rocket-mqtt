#[derive(Serialize, Deserialize)]
pub struct Hero {
  pub id: Option<i32>,
  pub name: &'static str,
  pub identity: &'static str,
  pub hometown: &'static str,
  pub age: i32
}
